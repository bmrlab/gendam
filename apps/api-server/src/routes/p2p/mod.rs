mod find_all;
pub mod info;

pub use crate::routes::p2p::info::ShareInfo;
use crate::CtxWithLibrary;
use rspc::{Router, RouterBuilder};
use serde::{Deserialize, Serialize};
use serde_json::json;
use specta::Type;
use uuid::Uuid;

pub fn get_routes<TCtx>() -> RouterBuilder<TCtx>
where
    TCtx: CtxWithLibrary + Clone + Send + Sync + 'static,
{
    Router::<TCtx>::new()
        .query("state", |t| {
            t(|ctx, _input: ()| async move {
                let node = ctx.node()?;
                node.state().await.map_err(|error| {
                    return rspc::Error::new(
                        rspc::ErrorCode::InternalServerError,
                        format!("invalid {:?}", error),
                    );
                })
            })
        })
        .mutation("share", |t| {
            #[derive(Deserialize, Type, Debug)]
            #[serde(rename_all = "camelCase")]
            struct SharePayload {
                file_id_list: Vec<i32>,
                peer_id: String,
            }
            t(|ctx, input: SharePayload| async move {
                tracing::debug!("start file share to {:#?}", input.peer_id);

                // 如果文件数量为空就不返回
                if input.file_id_list.is_empty() {
                    return Err(rspc::Error::new(
                        rspc::ErrorCode::BadRequest,
                        format!("file_id_list is empty"),
                    ));
                };

                let library = ctx.library().expect("failed load library").clone();

                let file_id_list = input.file_id_list.clone();

                let file_hashes = find_all::find_all_asset_object_hashes(
                    file_id_list,
                    library.clone().prisma_client(),
                )
                .await?;

                let temp_bundle_path = ctx.get_temp_dir().join(Uuid::new_v4().to_string());
                tracing::debug!("temp_bundle_path: {temp_bundle_path:?}");

                library
                    .generate_bundle(&file_hashes, &temp_bundle_path)
                    .map_err(|e| {
                        rspc::Error::new(
                            rspc::ErrorCode::InternalServerError,
                            format!("generate_bundle error: {e:?}"),
                        )
                    })?;

                let share_info = ShareInfo {
                    file_count: file_hashes.len(),
                };

                let node = ctx.node()?;

                match node
                    .start_share(
                        &input.peer_id,
                        vec![temp_bundle_path.clone()],
                        share_info,
                        move || async move {
                            if let Err(e) = tokio::fs::remove_file(&temp_bundle_path).await {
                                tracing::error!("failed to remove temp file: {e}");
                            }
                        },
                    )
                    .await
                {
                    Ok(id) => Ok(json!({"id": id})),
                    Err(e) => Err(rspc::Error::new(
                        rspc::ErrorCode::InternalServerError,
                        format!("start_share error: {e:?}"),
                    )),
                }
            })
        })
        .mutation("accept_file_share", |t| {
            #[derive(Serialize, Type)]
            #[serde(rename_all = "camelCase")]
            struct AcceptShareOutput {
                pub file_list: Vec<String>,
            }
            t(|ctx, id: Uuid| async move {
                let node = ctx.node()?;

                match node.get_payload(id) {
                    Ok(Some(payload)) => {
                        // 对于每个接收到的文件都创建一个对应的存放路径
                        let mut accept_file_path_list = vec![];
                        payload.file_list.iter().for_each(|_| {
                            accept_file_path_list
                                .push(ctx.get_temp_dir().join(Uuid::new_v4().to_string()));
                        });

                        tracing::debug!("accept_file_path_list: {accept_file_path_list:?}");

                        match node.accept_share(id, accept_file_path_list.clone()).await {
                            Ok(_) => (),
                            Err(error) => {
                                return Err(rspc::Error::new(
                                    rspc::ErrorCode::InternalServerError,
                                    format!("accept_share error: {error:?}"),
                                ))
                            }
                        }

                        Ok(AcceptShareOutput {
                            file_list: accept_file_path_list
                                .iter()
                                .map(|v| v.to_string_lossy().to_string())
                                .collect(),
                        })
                    }
                    _ => Err(rspc::Error::new(
                        rspc::ErrorCode::InternalServerError,
                        format!("accept_share error: payload not found"),
                    )),
                }
            })
        })
        .mutation("reject_file_share", |t| {
            t(|ctx, id: Uuid| async move {
                match ctx.node()?.reject_share(id).await {
                    Ok(_) => Ok(json!({ "status": "ok"})),
                    Err(error) => Err(rspc::Error::new(
                        rspc::ErrorCode::InternalServerError,
                        format!("reject_share error: {error:?}"),
                    )),
                }
            })
        })
        .mutation("cancel_file_share", |t| {
            t(|ctx, id: Uuid| async move {
                match ctx.node()?.cancel_share(id).await {
                    Ok(_) => Ok(json!({ "status": "ok"})),
                    Err(error) => Err(rspc::Error::new(
                        rspc::ErrorCode::InternalServerError,
                        format!("cancel_share error: {error:?}"),
                    )),
                }
            })
        })
        .mutation("finish_file_share", |t| {
            t(|ctx, file_path: String| async move {
                let library = ctx.library()?;
                match library.unpack_bundle(&file_path) {
                    Ok(file_hashes) => Ok(file_hashes),
                    Err(e) => {
                        tracing::error!("failed to unpack bundle ({file_path}): {e}");
                        Ok(vec![])
                    }
                }
            })
        })
        .subscription("events", |t| {
            t(|ctx, _input: ()| {
                let node = ctx.node().expect("failed load node on subscription events");

                let mut rx = node.events.subscribe();

                return async_stream::stream! {
                    while let Ok(event) = rx.recv().await {
                        tracing::info!("p2p recv event: {:#?}", event);
                        match event {
                            p2p::Event::ShareRequest {
                                id,
                                peer_id,
                                peer_name,
                                file_list,
                                share_info
                            } => {
                                yield json!({
                                    "type": "ShareRequest",
                                    "id": id,
                                    "peerId": peer_id,
                                    "peerName": peer_name,
                                    "shareInfo": share_info,
                                    "fileList": file_list,
                                });
                            }
                            p2p::Event::ShareProgress { id, percent, share_info } => {
                                yield json!({
                                    "type": "ShareProgress",
                                    "id": id,
                                    "percent": percent,
                                    "shareInfo": share_info
                                });
                            }
                            p2p::Event::ShareTimedOut { id } => {
                                yield json!({
                                    "type": "ShareTimedOut",
                                    "id": id,
                                });
                            }
                            p2p::Event::ShareRejected { id } => {
                                yield json!({
                                    "type": "ShareRejected",
                                    "id": id,
                                });
                            }
                        }
                    }
                };
            })
        })
}
