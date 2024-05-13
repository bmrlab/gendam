use crate::file_handler::{create_file_handler_task, get_file_handler};
use crate::CtxWithLibrary;
use prisma_client_rust::{operator, Direction};
use prisma_lib::PrismaClient;
use prisma_lib::{asset_object, file_handler_task, media_data};
use rspc::{Router, RouterBuilder};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::sync::Arc;

#[derive(Deserialize, Serialize, Type, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Pagination {
    // https://github.com/oscartbeaumont/rspc/issues/93
    page_size: i32,
    page_index: i32,
}

#[derive(Deserialize, Type, Debug)]
#[serde(rename_all = "camelCase")]
pub enum VideoTaskListRequestFilter {
    All,
    Processing,
    Completed,
    Failed,
    Canceled,
    ExcludeCompleted,
    ExitCode(i32),
}

#[derive(Deserialize, Type, Debug)]
#[serde(rename_all = "camelCase")]
struct VideoTaskListRequestPayload {
    pagination: Pagination,
    filter: VideoTaskListRequestFilter,
}

#[derive(Deserialize, Type, Debug)]
#[serde(rename_all = "camelCase")]
struct TaskCancelRequestPayload {
    asset_object_id: i32,
    task_types: Option<Vec<String>>,
}

#[derive(Deserialize, Type, Debug)]
#[serde(rename_all = "camelCase")]
struct TaskRedoRequestPayload {
    asset_object_id: i32,
}

#[derive(Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct VideoWithTasksResult {
    pub name: String,
    pub materialized_path: String,
    pub asset_object: asset_object::Data,
    pub tasks: Vec<file_handler_task::Data>,
    pub media_data: Option<media_data::Data>,
}

#[derive(Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct VideoWithTasksPageResult {
    data: Vec<VideoWithTasksResult>,
    pagination: Pagination,
    max_page: i32,
}

struct VideoTaskHandler {
    prisma_client: Arc<PrismaClient>,
}

impl VideoTaskHandler {
    fn new(prisma_client: Arc<PrismaClient>) -> Self {
        VideoTaskHandler { prisma_client }
    }

    async fn count(&self, task_filter: Vec<file_handler_task::WhereParam>) -> anyhow::Result<i64> {
        let count = self
            .prisma_client
            .asset_object()
            .count(vec![asset_object::tasks::some(task_filter)])
            .exec()
            .await
            .map_err(|e| anyhow::anyhow!(e))?;
        Ok(count)
    }

    async fn get_max_page(
        &self,
        page_size: i32,
        task_filter: Vec<file_handler_task::WhereParam>,
    ) -> anyhow::Result<i32> {
        let count = self.count(task_filter).await?;
        Ok((count as f64 / page_size as f64).ceil() as i32)
    }

    async fn list(
        &self,
        payload: VideoTaskListRequestPayload,
    ) -> anyhow::Result<VideoWithTasksPageResult> {
        let Pagination {
            page_size,
            mut page_index,
        } = payload.pagination;
        if page_index < 1 {
            page_index = 1;
        }

        let task_filter = match payload.filter {
            VideoTaskListRequestFilter::ExcludeCompleted => vec![operator::or(vec![
                file_handler_task::exit_code::equals(None),
                file_handler_task::exit_code::gte(1),
            ])],
            _ => vec![],
        };

        let max_page = self
            .get_max_page(payload.pagination.page_size, task_filter.clone())
            .await?;

        let asset_object_data_list = self
            .prisma_client
            .asset_object()
            .find_many(vec![asset_object::tasks::some(task_filter)])
            .with(asset_object::tasks::fetch(vec![]))
            .with(asset_object::file_paths::fetch(vec![]))
            // bindings 中不会自动生成 media_data 类型
            .with(asset_object::media_data::fetch())
            .order_by(asset_object::created_at::order(Direction::Desc))
            .skip((page_size * (page_index - 1)).into())
            .take(page_size.into())
            .exec()
            .await
            .expect("failed to list video tasks");

        let videos_with_tasks = asset_object_data_list
            .iter()
            .map(|asset_object_data| {
                let (materialized_path, name) = match asset_object_data.file_paths {
                    Some(ref file_paths) => {
                        if file_paths.len() > 0 {
                            let file_path = file_paths[0].clone();
                            (file_path.materialized_path.clone(), file_path.name.clone())
                        } else {
                            ("".to_string(), "".to_string())
                        }
                    }
                    None => ("".to_string(), "".to_string()),
                };
                let mut asset_object_data = asset_object_data.to_owned();
                let tasks = asset_object_data.tasks.unwrap_or(vec![]);
                asset_object_data.tasks = None;
                let media_data = asset_object_data
                    .media_data
                    .take()
                    .map(|data| data.map(|d| *d));
                VideoWithTasksResult {
                    name,
                    materialized_path,
                    asset_object: asset_object_data,
                    media_data: media_data.unwrap_or(None),
                    tasks,
                }
            })
            .collect::<Vec<VideoWithTasksResult>>();
        Ok(VideoWithTasksPageResult {
            data: videos_with_tasks,
            pagination: payload.pagination,
            max_page,
        })
    }

    pub async fn cancel(
        &self,
        input: TaskCancelRequestPayload,
        ctx: &impl CtxWithLibrary,
    ) -> anyhow::Result<()> {
        let asset_object_id = input.asset_object_id;

        let tx = ctx.task_tx()?;
        match input.task_types.as_ref() {
            Some(task_types) => task_types.iter().for_each(|t| {
                if let Err(e) = tx.send(crate::file_handler::TaskPayload::CancelByAssetAndType(
                    asset_object_id,
                    t.to_string(),
                )) {
                    tracing::warn!("cancel task({}-{}) error: {}", asset_object_id, t, e);
                }
            }),
            _ => {
                // cancel all tasks
                tx.send(crate::file_handler::TaskPayload::CancelByAssetId(
                    asset_object_id,
                ))?;
            }
        }

        let mut filter = vec![
            file_handler_task::asset_object_id::equals(asset_object_id),
            // 将所有未完成的任务设置为取消
            file_handler_task::exit_code::equals(None),
        ];

        if let Some(task_types) = input.task_types {
            // 如果传入了任务类型，则需要进一步过滤
            filter.push(file_handler_task::task_type::in_vec(task_types));
        }

        self.prisma_client
            .file_handler_task()
            .update_many(filter, vec![file_handler_task::exit_code::set(Some(1))])
            .exec()
            .await
            .map_err(|e| anyhow::anyhow!(e))?;
        Ok(())
    }

    async fn regenerate(
        &self,
        payload: TaskRedoRequestPayload,
        ctx: &impl CtxWithLibrary,
    ) -> anyhow::Result<()> {
        let asset_object_data = self
            .prisma_client
            .asset_object()
            .find_unique(asset_object::id::equals(payload.asset_object_id))
            .exec()
            .await?;

        if let Some(asset_object_data) = asset_object_data {
            let handler = get_file_handler(&asset_object_data, ctx)?;
            for task_type in handler.get_supported_task_types() {
                if let Err(e) = handler.delete_artifacts_by_task(task_type.0.as_str()).await {
                    tracing::warn!(
                        "delete_artifacts_by_task({}) error: {}",
                        task_type.0,
                        e
                    );
                }
            }

            create_file_handler_task(&asset_object_data, ctx, None, None)
                .await
                .map_err(|e| anyhow::anyhow!("failed to create video task: {e:?}"))?;
        }
        Ok(())
    }
}

pub fn get_routes<TCtx>() -> RouterBuilder<TCtx>
where
    TCtx: CtxWithLibrary + Clone + Send + Sync + 'static,
{
    Router::<TCtx>::new()
        .mutation("create", |t| {
            t(|_ctx: TCtx, video_path: String| {
                if video_path.is_empty() {
                    return Err(rspc::Error::new(
                        rspc::ErrorCode::InternalServerError,
                        String::from("this api is deprecated"),
                    ));
                } else {
                    return Ok(());
                }
            })
        })
        .query("list", |t| {
            t(|ctx: TCtx, input: VideoTaskListRequestPayload| async move {
                let library = ctx.library()?;
                match VideoTaskHandler::new(library.prisma_client())
                    .list(input)
                    .await
                {
                    Ok(res) => Ok(res),
                    Err(e) => Err(rspc::Error::new(
                        rspc::ErrorCode::InternalServerError,
                        format!("task list failed: {}", e),
                    )),
                }
            })
        })
        .mutation("regenerate", |t| {
            t(|ctx: TCtx, input: TaskRedoRequestPayload| async move {
                let library = ctx.library()?;
                VideoTaskHandler::new(library.prisma_client())
                    .regenerate(input, &ctx)
                    .await
                    .map_err(|e| {
                        rspc::Error::new(
                            rspc::ErrorCode::InternalServerError,
                            format!("task regenerate failed: {}", e),
                        )
                    })?;
                Ok(())
            })
        })
        .mutation("cancel", |t| {
            t(|ctx: TCtx, input: TaskCancelRequestPayload| async move {
                let library = ctx.library()?;
                VideoTaskHandler::new(library.prisma_client())
                    .cancel(input, &ctx)
                    .await
                    .map_err(|e| {
                        rspc::Error::new(
                            rspc::ErrorCode::InternalServerError,
                            format!("task cancel failed: {}", e),
                        )
                    })?;
                Ok(())
            })
        })
}
/*
        .procedure(
            "create_video_frames",
            R.mutation(|ctx, video_path: String| async move {
                let res = create_video_frames(&ctx, &video_path).await;
                serde_json::to_value(res).unwrap()
            })
        )
*/

// async fn create_video_frames(ctx: &Ctx, video_path: &str) {
//     let video_handler =
//         file_handler::video::VideoHandler::new(
//             video_path,
//             &ctx.local_data_dir,
//             &ctx.resources_dir,
//         )
//         .await
//         .expect("failed to initialize video handler");
//     let frame_handle = tokio::spawn(async move {
//         match video_handler.get_frames().await {
//             Ok(res) => {
//                 debug!("successfully got frames");
//                 Ok(res)
//             },
//             Err(e) => {
//                 debug!("failed to get frames: {}", e);
//                 Err(e)
//             }
//         }
//     });
//     let result = frame_handle.await.unwrap();
//     result.expect("failed to get frames");
// }
//
