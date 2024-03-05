use crate::CtxWithLibrary;
use rspc::{Rspc, Router};
use prisma_client_rust::Direction;
use prisma_lib::{new_client_with_url, video_task};
use serde::Serialize;
use serde_json::json;
use specta::Type;
use crate::task_queue::create_video_task;

pub fn get_routes<TCtx>() -> Router<TCtx>
where TCtx: CtxWithLibrary + Clone + Send + Sync + 'static
{
    Rspc::<TCtx>::new().router()
        .procedure(
            "create",
            Rspc::<TCtx>::new().mutation(move |ctx: TCtx, video_path: String| {
                let tx = ctx.get_task_tx();
                async move {
                    if let Ok(res) = create_video_task(&ctx, &video_path, tx, ctx.get_index()).await {
                        return serde_json::to_value(res).unwrap();
                    } else {
                        return json!({
                            "error": "failed to create video task"
                        });
                    }
                }
            }),
        )
        .procedure(
            "list",
            Rspc::<TCtx>::new().query(move |ctx: TCtx, _input: ()| async move {
                let library = ctx.load_library();
                let client = new_client_with_url(library.db_url.as_str())
                    .await
                    .expect("failed to create prisma client");

                let res = client
                    .video_task()
                    .find_many(vec![])
                    .order_by(video_task::id::order(Direction::Desc))
                    .exec()
                    .await
                    .expect("failed to list video tasks");

                #[derive(Serialize, Type)]
                pub struct VideoTaskResult {
                    #[serde(rename = "id")]
                    pub id: i32,
                    #[serde(rename = "videoPath")]
                    pub video_path: String,
                    #[serde(rename = "videoFileHash")]
                    pub video_file_hash: String,
                    #[serde(rename = "taskType")]
                    pub task_type: String,
                    #[serde(rename = "startsAt")]
                    pub starts_at: Option<String>,
                    #[serde(rename = "endsAt")]
                    pub ends_at: Option<String>,
                }

                res.iter()
                    .map(|item| VideoTaskResult {
                        id: item.id,
                        video_path: item.video_path.clone(),
                        video_file_hash: item.video_file_hash.clone(),
                        task_type: item.task_type.to_string(),
                        starts_at: if let Some(t) = item.starts_at {
                            Some(t.to_string())
                        } else {
                            None
                        },
                        ends_at: if let Some(t) = item.ends_at {
                            Some(t.to_string())
                        } else {
                            None
                        },
                    })
                    .collect::<Vec<_>>()
                // serde_json::to_value(res).unwrap()
            }),
        )
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
