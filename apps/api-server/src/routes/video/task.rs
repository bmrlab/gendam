use crate::CtxWithLibrary;
use rspc::{Rspc, Router};
use prisma_client_rust::Direction;
use prisma_lib::{asset_object, file_handler_task};
use serde::Serialize;
// use serde_json::json;
use specta::Type;
// use crate::task_queue::create_video_task;

pub fn get_routes<TCtx>() -> Router<TCtx>
where TCtx: CtxWithLibrary + Clone + Send + Sync + 'static
{
    Rspc::<TCtx>::new().router()
        .procedure(
            "create",
            Rspc::<TCtx>::new().mutation(move |_ctx: TCtx, video_path: String| {
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
            }),
        )
        .procedure(
            "list",
            Rspc::<TCtx>::new().query(move |ctx: TCtx, _input: ()| async move {
                let library = ctx.library()?;
                let asset_object_data_list = library.prisma_client()
                    .asset_object()
                    .find_many(vec![])
                    .with(asset_object::tasks::fetch(vec![]))
                    .with(asset_object::file_paths::fetch(vec![]))
                    .order_by(asset_object::id::order(Direction::Desc))
                    .exec()
                    .await
                    .expect("failed to list video tasks");

                #[derive(Serialize, Type)]
                #[serde(rename_all = "camelCase")]
                pub struct VideoTaskResult {
                    pub task_type: String,
                    pub starts_at: Option<String>,
                    pub ends_at: Option<String>,
                }

                #[derive(Serialize, Type)]
                #[serde(rename_all = "camelCase")]
                pub struct VideoWithTasksResult {
                    pub name: String,
                    pub materialized_path: String,
                    pub asset_object_id: i32,
                    pub asset_object_hash: String,
                    pub tasks: Vec<VideoTaskResult>,
                }

                let videos_with_tasks = asset_object_data_list.iter()
                    .map(|asset_object_data| {
                        let (materialized_path, name) = match asset_object_data.file_paths {
                            Some(ref file_paths) => {
                                if file_paths.len() > 0 {
                                    let file_path = file_paths[0].clone();
                                    (file_path.materialized_path.clone(), file_path.name.clone())
                                } else {
                                    ("".to_string(), "".to_string())
                                }
                            },
                            None => ("".to_string(), "".to_string()),
                        };
                        let asset_object_hash = asset_object_data.hash.clone().unwrap_or("".to_string());
                        let asset_object_id = asset_object_data.id;
                        let tasks =
                            asset_object_data.tasks.as_ref()
                            .unwrap_or(&vec![])
                            .iter()
                            .map(|file_handler_task::Data { task_type, starts_at, ends_at, .. }| {
                                VideoTaskResult {
                                    task_type: task_type.clone(),
                                    starts_at: match starts_at {
                                        Some(starts_at) => Some(starts_at.to_string()),
                                        None => None,
                                    },
                                    ends_at: match ends_at {
                                        Some(ends_at) => Some(ends_at.to_string()),
                                        None => None,
                                    },
                                }
                            })
                            .collect::<Vec<VideoTaskResult>>();
                        VideoWithTasksResult {
                            name,
                            materialized_path,
                            asset_object_id,
                            asset_object_hash,
                            tasks,
                        }
                    })
                    .collect::<Vec<VideoWithTasksResult>>();
                Ok(videos_with_tasks)
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
