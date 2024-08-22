use crate::content_metadata::ContentMetadataWithType;
use crate::CtxWithLibrary;
use content_base::task::CancelTaskPayload;
use content_base::upsert::UpsertPayload;
use content_base::ContentMetadata;
use prisma_client_rust::{operator, Direction};
use prisma_lib::PrismaClient;
use prisma_lib::{asset_object, file_handler_task};
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
    pub media_data: Option<ContentMetadataWithType>,
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
}

pub fn get_routes<TCtx>() -> RouterBuilder<TCtx>
where
    TCtx: CtxWithLibrary + Clone + Send + Sync + 'static,
{
    Router::<TCtx>::new().mutation("create", |t| {
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
