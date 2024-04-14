use crate::task_queue::create_video_task;
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
pub enum TaskListRequestFilter {
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
struct TaskListRequestPayload {
    pagination: Pagination,
    filter: TaskListRequestFilter,
}

#[derive(Deserialize, Type, Debug)]
#[serde(rename_all = "camelCase")]
struct TaskCancelRequestPayload {
    asset_object_id: i32,
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
        payload: TaskListRequestPayload,
    ) -> anyhow::Result<VideoWithTasksPageResult> {
        let Pagination {
            page_size,
            mut page_index,
        } = payload.pagination;
        if page_index < 1 {
            page_index = 1;
        }

        let task_filter = match payload.filter {
            TaskListRequestFilter::ExcludeCompleted => vec![operator::or(vec![
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

        let tx = ctx.get_task_tx();

        if let Ok(tx) = tx.lock() {
            tx.send(crate::task_queue::TaskPayload::CancelByAssetId(
                asset_object_id,
            ))?;
        }

        self.prisma_client
            .file_handler_task()
            .update_many(
                vec![
                    file_handler_task::asset_object_id::equals(asset_object_id),
                    file_handler_task::starts_at::equals(None),
                ],
                vec![file_handler_task::exit_code::set(Some(1))],
            )
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
            .with(asset_object::media_data::fetch())
            .exec()
            .await?;
        if let Some(asset_object_data) = asset_object_data {
            create_video_task(&asset_object_data, ctx, None)
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
                // let tx = ctx.get_task_tx();
                // async move {
                //     if let Ok(res) = create_video_task(&ctx, &video_path, tx).await {
                //         return serde_json::to_value(res).unwrap();
                //     } else {
                //         return json!({
                //             "error": "failed to create video task"
                //         });
                //     }
                // }
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
            t(|ctx: TCtx, input: TaskListRequestPayload| async move {
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
        .mutation("trigger_unfinished", |t| {
            t(|ctx: TCtx, library_id: String| async move {
                /*
                 * TODO 如果这里不加一个参数直接用 _input: (), 会因参数校验失败而返回错误,
                 * 因为前端会发一个 payload: `{}`, 而不是空, 这个 issue 需要排查一下
                 * 现在就索性校验一下 library_id, 实际没啥用
                 */
                let library = ctx.library()?;
                if library.id != library_id {
                    return Err(rspc::Error::new(
                        rspc::ErrorCode::BadRequest,
                        String::from("The library is not the current library"),
                    ));
                }
                ctx.trigger_unfinished_tasks().await;
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
