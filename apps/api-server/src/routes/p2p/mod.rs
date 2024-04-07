mod generated;
mod utils;

use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, PoisonError};
use std::time::Duration;

use crate::routes::p2p::utils::create_requests::create_requests;
use crate::routes::p2p::utils::find_all_path::find_all_path;
use crate::CtxWithLibrary;
use futures::{AsyncReadExt, AsyncWriteExt};
use p2p::{str_to_peer_id, FilePath};
use p2p_block::Transfer;
use p2p_block::{message::Message, BlockSize, SpaceblockRequests};
use rspc::{Router, RouterBuilder};
use serde::Deserialize;
use serde_json::json;
use specta::Type;
use tokio::fs::File;
use tokio::io::BufReader;
use tokio::time::{sleep, Instant};
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

                // 临时文件夹
                let temp_dir = ctx.get_temp_dir();

                let file_paths =
                    find_all_path(file_id_list, library.clone().prisma_client()).await?;

                tracing::info!("file_paths: {:#?}", file_paths);

                let files: Vec<(
                    String,
                    String,
                    std::path::PathBuf,
                    std::path::PathBuf,
                    String,
                    String,
                )> = file_paths
                    .iter()
                    .map(|file_path| {
                        let name = file_path.name.clone();
                        let hash = file_path.hash.clone();
                        let path = file_path.path.clone();
                        let files_dir = library.file_path(&hash);
                        let artifacts_dir = library.artifacts_dir(&hash);
                        let artifacts_dir_zip_path = format!("{}/{}.zip", temp_dir.display(), hash);
                        (
                            name,
                            hash,
                            files_dir,
                            artifacts_dir,
                            artifacts_dir_zip_path,
                            path,
                        )
                    })
                    .into_iter()
                    .collect::<Vec<_>>();

                tracing::debug!("files: {:#?}", files);

                let (files, requests): (Vec<_>, Vec<_>) = create_requests(files).await?;

                // 总共要发送的大小
                let total_length: u64 = requests
                    .iter()
                    .map(|req| req.size + req.artifact_size)
                    .sum();

                tracing::debug!("total_length: {:#?}", total_length);

                // 这次任务id
                let id = Uuid::new_v4();

                let peer_id = match str_to_peer_id(input.peer_id) {
                    Ok(peer_id) => peer_id,
                    Err(error) => {
                        return Err(rspc::Error::new(
                            rspc::ErrorCode::BadRequest,
                            format!("str_to_peer_id error: {error:?}"),
                        ))
                    }
                };

                let node = ctx.node()?;

                let peer = match node.get_peers().get(&peer_id) {
                    Some(peer) => peer.clone(),
                    None => {
                        return Err(rspc::Error::new(
                            rspc::ErrorCode::BadRequest,
                            "peer no found".to_string(),
                        ))
                    }
                };

                tracing::debug!("starting file share with {peer:#?}");

                tracing::debug!("({id}): starting Spacedrop with peer '{peer_id}, {total_length}");

                // 开启stream
                let mut stream = match node.open_stream(peer_id).await {
                    Ok(stream) => stream,
                    Err(error) => {
                        return Err(rspc::Error::new(
                            rspc::ErrorCode::InternalServerError,
                            format!("open stream error: {error:?}"),
                        ))
                    }
                };

                tracing::debug!("open stream: {:#?}", stream);

                // 提交线程池
                tokio::spawn(async move {
                    tracing::debug!("({id}): connected, sending message");
                    let message = Message::Share(SpaceblockRequests {
                        id,
                        block_size: BlockSize::from_size(total_length),
                        requests,
                    });

                    tracing::info!("sending message: {:#?}", message);

                    let buf = &message.to_bytes();

                    // tracing::info!("message {buf:#?}, len :{:#?}", buf.len());

                    // 写入stream
                    if let Err(err) = stream.write_all(buf).await {
                        tracing::error!("({id}): failed to send header: {err}");
                        return;
                    }

                    let Message::Share(requests) = message;

                    tracing::debug!("({id}): waiting for response");

                    let mut buf = [0u8; 1];
                    // 结果
                    let _ = tokio::select! {
                        result = stream.read_exact(&mut buf) => result,
                        _ = sleep(Duration::from_secs(60) + Duration::from_secs(5)) => {
                            // 65秒超时
                            tracing::debug!("({id}): timed out, cancelling");
                            // todo websocket 发给前端 超时了
                            return;
                        },
                    }
                    .expect("read_exact fail");

                    tracing::info!("({id}): received response: {buf:?}");

                    match buf[0] {
                        0 => {
                            tracing::debug!("({id}): Spacedrop was rejected from peer '{peer_id}'");
                            // todo websocket 发给前端 被拒绝了
                            node.events.send(p2p::Event::ShareRejected { id }).ok();
                            return;
                        }
                        1 => {}       // Okay
                        _ => todo!(), // TODO: Proper error
                    }

                    let cancelled = Arc::new(AtomicBool::new(false));
                    node.spacedrop_cancellations
                        .lock()
                        .unwrap_or_else(PoisonError::into_inner)
                        .insert(id, cancelled.clone());

                    tracing::info!("({id}): starting transfer");

                    // 记录开始时间
                    let i = Instant::now();

                    let mut transfer = Transfer::new(
                        &requests,
                        |percent| {
                            node.events
                                .send(p2p::Event::ShareProgress { id, percent })
                                .ok();
                            tracing::info!("({id}): progress: {percent}%");
                        },
                        &cancelled,
                    );

                    for (file_id, (path, file, artifact_zip_path)) in files.into_iter().enumerate()
                    {
                        tracing::debug!("({id}): transmitting '{file_id}' from '{path:?}'");

                        let file = BufReader::new(file);

                        let artifact_zip_file = File::open(Path::new(&artifact_zip_path))
                            .await
                            .expect("没有找到压缩文件");

                        let artifact_zip_file_buf = BufReader::new(artifact_zip_file);

                        tracing::debug!("artifact_zip_file_buf {:#?}", artifact_zip_file_buf);

                        // 先发送文件
                        if let Err(err) = transfer
                            .send(&mut stream, file, artifact_zip_file_buf)
                            .await
                        {
                            tracing::error!("({id}): failed to send file: {err}");
                            // TODO: Error to frontend
                            // p2p.events
                            // 	.send(P2PEvent::SpacedropFailed { id, file_id })
                            // 	.ok();
                            // todo!("向前端websocket发送错误");
                            return;
                        }

                        // 删除压缩文件
                        tokio::fs::remove_file(artifact_zip_path)
                            .await
                            .expect("删除压缩文件失败");
                    }

                    // 传输完成
                    tracing::debug!("({id}): finished; took '{:?}", i.elapsed());

                    // 下个任务是发送 crdt的 同步信息
                    // todo!("发送crdt的同步信息");
                });

                Ok(json!({"id": id}))
            })
        })
        .mutation("acceptFileShare", |t| {
            t(|ctx, (id, hashes): (Uuid, Option<Vec<String>>)| async move {
                // todo 这个path 应该是数据库的地址， 真实保存的地址是根据hash来的
                let node = ctx.node()?;

                match hashes {
                    Some(hashes) => {
                        // 真实路径
                        let local_data_root = ctx.get_local_data_root();

                        let library_id = ctx.library()?.id;

                        let mut file_paths = Vec::new();

                        for hash in hashes {
                            let local_data_root = local_data_root.display();
                            let data_path_string = format!(
                                "{}/libraries/{}/files/{}/{}",
                                local_data_root,
                                library_id,
                                &hash[0..3],
                                hash
                            );

                            let artifact_path_string = format!(
                                "{}/libraries/{}/artifacts/{}/{}.zip",
                                local_data_root,
                                library_id,
                                &hash[0..3],
                                hash
                            );

                            file_paths.push(FilePath {
                                hash,
                                file_path: data_path_string,
                                artifact_path: artifact_path_string,
                            })
                        }

                        match node.accept_share(id, file_paths).await {
                            Ok(_) => (),
                            Err(error) => {
                                return Err(rspc::Error::new(
                                    rspc::ErrorCode::InternalServerError,
                                    format!("accept_share error: {error:?}"),
                                ))
                            }
                        }
                    }
                    None => match node.reject_share(id).await {
                        Ok(_) => (),
                        Err(error) => {
                            return Err(rspc::Error::new(
                                rspc::ErrorCode::InternalServerError,
                                format!("reject_share error: {error:?}"),
                            ))
                        }
                    },
                };

                Ok(json!({ "status": "ok"}))
            })
        })
        .mutation("cancelFileShare", |t| {
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
                                files,
                            } => {
                                yield json!({
                                    "type": "ShareRequest",
                                    "id": id,
                                    "peerId": peer_id,
                                    "peerName": peer_name,
                                    "files": files,
                                });
                            }
                            p2p::Event::ShareProgress { id, percent } => {
                                yield json!({
                                    "type": "ShareProgress",
                                    "id": id,
                                    "percent": percent,
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
