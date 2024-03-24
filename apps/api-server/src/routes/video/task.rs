use crate::task_queue::create_video_task;
use crate::CtxWithLibrary;
use prisma_client_rust::Direction;
use prisma_lib::{asset_object, file_handler_task};
use rspc::{Router, RouterBuilder};
use serde::{Deserialize, Serialize};
use specta::Type;

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
            #[derive(Serialize, Type)]
            #[serde(rename_all = "camelCase")]
            pub struct VideoTaskResult {
                pub task_type: String,
                pub starts_at: Option<String>,
                pub ends_at: Option<String>,
            }

            #[derive(Serialize, Type)]
            #[serde(rename_all = "camelCase")]
            pub struct MediaDataResult {
                pub id: i32,
                pub width: i32,
                pub height: i32,
                pub duration: i32,
                pub bit_rate: i32,
                pub size: i32,
            }

            #[derive(Serialize, Type)]
            #[serde(rename_all = "camelCase")]
            pub struct AssetObjectResult {
                pub id: i32,
                pub hash: String,
                pub media_data: Option<MediaDataResult>,
            }

            #[derive(Serialize, Type)]
            #[serde(rename_all = "camelCase")]
            pub struct VideoWithTasksResult {
                pub name: String,
                pub materialized_path: String,
                // pub asset_object_id: i32,
                // pub asset_object_hash: String,
                pub asset_object: AssetObjectResult,
                pub tasks: Vec<VideoTaskResult>,
            }

            t(|ctx: TCtx, _input: ()| async move {
                let library = ctx.library()?;
                let asset_object_data_list = library
                    .prisma_client()
                    .asset_object()
                    .find_many(vec![])
                    .with(asset_object::tasks::fetch(vec![]))
                    .with(asset_object::file_paths::fetch(vec![]))
                    .with(asset_object::media_data::fetch())
                    .order_by(asset_object::id::order(Direction::Desc))
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
                        let tasks = asset_object_data
                            .tasks
                            .as_ref()
                            .unwrap_or(&vec![])
                            .iter()
                            .map(
                                |file_handler_task::Data {
                                     task_type,
                                     starts_at,
                                     ends_at,
                                     ..
                                 }| {
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
                                },
                            )
                            .collect::<Vec<VideoTaskResult>>();
                        VideoWithTasksResult {
                            name,
                            materialized_path,
                            asset_object: AssetObjectResult {
                                id: asset_object_data.id,
                                hash: asset_object_data.hash.clone(),
                                media_data: match asset_object_data.media_data {
                                    Some(ref media_data) => match media_data {
                                        Some(ref media_data) => Some(MediaDataResult {
                                            id: media_data.id,
                                            width: media_data.width.unwrap_or_default(),
                                            height: media_data.height.unwrap_or_default(),
                                            duration: media_data.duration.unwrap_or_default(),
                                            bit_rate: media_data.bit_rate.unwrap_or_default(),
                                            size: media_data.size.unwrap_or_default(),
                                        }),
                                        None => None,
                                    },
                                    None => None,
                                },
                            },
                            tasks,
                        }
                    })
                    .collect::<Vec<VideoWithTasksResult>>();
                Ok(videos_with_tasks)
            })
        })
        .mutation("regenerate", |t| {
            #[derive(Deserialize, Type, Debug)]
            #[serde(rename_all = "camelCase")]
            struct TaskRegeneratePayload {
                materialized_path: String,
                asset_object_id: i32,
            }
            t(|ctx: TCtx, input: TaskRegeneratePayload| async move {
                let library = ctx.library()?;
                let asset_object_data  = library
                    .prisma_client()
                    .asset_object()
                    .find_unique(asset_object::id::equals(input.asset_object_id))
                    .exec()
                    .await
                    .map_err(|e| {
                        rspc::Error::new(
                            rspc::ErrorCode::InternalServerError,
                            format!("sql query failed: {}", e),
                        )
                    })?;
                if let Some(asset_object_data) = asset_object_data {
                    create_video_task(
                        &input.materialized_path,
                        &asset_object_data,
                        &ctx,
                        ctx.get_task_tx(),
                    )
                    .await
                    .map_err(|e| {
                        rspc::Error::new(
                            rspc::ErrorCode::NotFound,
                            format!("failed to create video task: {e:?}"),
                        )
                    })?;
                } else {
                    return Err(rspc::Error::new(
                        rspc::ErrorCode::NotFound,
                        format!("asset object not found"),
                    ));
                };

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
