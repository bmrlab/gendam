use crate::task_queue::create_video_task;
use crate::CtxWithLibrary;
use prisma_client_rust::Direction;
use prisma_lib::{asset_object, file_handler_task, media_data};
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
            pub struct VideoWithTasksResult {
                pub name: String,
                pub materialized_path: String,
                pub asset_object: asset_object::Data,
                pub tasks: Vec<file_handler_task::Data>,
                pub media_data: Option<media_data::Data>,
            }

            t(|ctx: TCtx, _input: ()| async move {
                let library = ctx.library()?;
                let asset_object_data_list = library
                    .prisma_client()
                    .asset_object()
                    .find_many(vec![])
                    .with(asset_object::tasks::fetch(vec![]))
                    .with(asset_object::file_paths::fetch(vec![]))
                    // bindings 中不会自动生成 media_data 类型
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
                let asset_object_data = library
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
        .mutation("cancel", |t| {
            #[derive(Deserialize, Type, Debug)]
            #[serde(rename_all = "camelCase")]
            struct CancelPayload {
                asset_object_id: i32,
            }
            t(|ctx: TCtx, input: CancelPayload| async move {
                let library = ctx.library()?;
                let asset_object_id = input.asset_object_id;
                library
                    .prisma_client()
                    .file_handler_task()
                    .update_many(
                        vec![file_handler_task::asset_object_id::equals(asset_object_id)],
                        vec![file_handler_task::exit_code::set(Some(1))],
                    )
                    .exec()
                    .await
                    .map_err(|e| {
                        rspc::Error::new(
                            rspc::ErrorCode::InternalServerError,
                            format!("sql query failed: {}", e),
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
