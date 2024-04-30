use std::sync::Arc;

use automerge::AutoSerde;
use autosurgeon::hydrate;
use content_library::Library;
use futures::AsyncWriteExt;
use p2p::{str_to_peer_id, Event, Node};
use p2p_block::SyncMessage;
use prisma_lib::asset_object;
use sync_lib::utils::str_to_document_id;
use uuid::Uuid;

use crate::{
    sync::{File as SyncFile, Folder},
    ShareInfo,
};

pub async fn handle_p2p_message(
    event: Event<ShareInfo>,
    node: Arc<Node<ShareInfo>>,
    mut sync: sync_lib::Sync,
    library: Arc<Library>,
) -> Result<(), anyhow::Error> {
    let _ = match event {
        // peer_id 的 doc_id 文档发生更改
        p2p::Event::Sync {
            id,
            peer_id,
            doc_id,
        } => {
            tracing::debug!("({id}) p2p event Sync doc id: {:?}", doc_id);
            sync.events
                .send(sync_lib::event::Event::DocumentChanged {
                    doc_id: doc_id.clone(),
                    peer_id: peer_id.to_base58(),
                })
                .ok();
        }
        // 收到这个就要把同步消息发送给对方，直到同步消息为none
        p2p::Event::SyncTransfer(SyncMessage {
            peer_id,
            doc_id,
            message,
        }) => {
            tracing::info!(
                "接收到了同步文档 doc id: {:?}, message: {:?}",
                doc_id,
                message
            );

            let doc = str_to_document_id(doc_id.clone())?;
            tracing::debug!("doc: {doc:?}");
            let mut state = sync.get_document_state(doc.clone(), peer_id.clone())?;
            tracing::debug!("state: {state:?}");
            
            // 加载文档
            let doc_handle = sync
                .request_document(doc.clone())
                .await
                .expect("load_document error");

            //序列化文档
            // 接收消息
            if let Some(message_ref) = message.clone() {
                sync.receive_sync_message(doc_handle.clone(), &mut state, message_ref)
                    .await
                    .expect("receive_sync_message error");
            }

            // 再生成消息
            let new_message = sync
                .generate_sync_message(doc_handle.clone(), &mut state)
                .await
                .expect("generate_sync_message error");
            tracing::debug!("再次生成的new_message : {new_message:?}");

            let _ = sync
                .save_document_state(doc.clone(), peer_id.clone(), state)
                .unwrap();

            if message.is_none() && new_message.is_none() {
                // 这里要判断上一次有没有发过空消息
                let has_message = sync
                    .get_prev_has_message(doc.clone(), peer_id.clone())
                    .unwrap();
                tracing::debug!("has_message: {has_message:?}");
                if !has_message {
                    let sync_message = SyncMessage {
                        doc_id: doc_id.clone(),
                        peer_id: peer_id.clone(),
                        message: new_message.clone(),
                    };

                    let p2p_message =
                        p2p_block::message::Message::<SyncMessage>::Sync(sync_message);
                    let bytes = p2p_message.to_bytes();
                    tracing::debug!("再次发送文档");
                    let mut stream = node
                        .open_stream(str_to_peer_id(peer_id.clone()).unwrap())
                        .await
                        .expect("open stream fail");
                    stream.write_all(&bytes).await.unwrap();
                    // 修改has_message
                    let _ = sync
                        .save_prev_has_message(doc.clone(), peer_id.clone(), true)
                        .unwrap();

                    // 这里是发送方触发的结束， 接收方也会触发
                    let value = doc_handle
                        .with_doc(|doc| serde_json::to_value(AutoSerde::from(doc)).unwrap());
                    tracing::info!("当前文档0: {:#?}", value);
                } else {
                    // 这里是接收方触发的结束
                    // 更新数据库
                    if let Ok(file) =
                        doc_handle.with_doc(|doc| hydrate::<automerge::Automerge, SyncFile>(doc))
                    {
                        file.save_to_db(library.clone(), doc_id.clone()).await?;
                    };

                    if let Ok(folder) =
                        doc_handle.with_doc(|doc| hydrate::<automerge::Automerge, Folder>(doc))
                    {
                        folder
                            .save_to_db(
                                library.clone(),
                                doc_id.clone(),
                                node.clone(),
                                peer_id.clone(),
                            )
                            .await?;
                    };
                }
            } else {
                // 再次发送消息
                let sync_message = SyncMessage {
                    doc_id: doc_id.clone(),
                    peer_id: peer_id.clone(),
                    message: new_message.clone(),
                };

                let p2p_message = p2p_block::message::Message::<SyncMessage>::Sync(sync_message);
                let bytes = p2p_message.to_bytes();
                tracing::debug!("再次发送文档");
                let mut stream = node
                    .open_stream(str_to_peer_id(peer_id.clone()).unwrap())
                    .await
                    .expect("open stream fail");
                stream.write_all(&bytes).await.unwrap();
            }
        }
        p2p::Event::SyncRequest {
            id,
            peer_id,
            doc_id,
        } => {
            tracing::info!("api server 收到了别人的请求事件 id: {id:?}");
            let peer = str_to_peer_id(peer_id.clone()).unwrap();
            if let Ok(mut stream) = node.open_stream(peer).await {
                let _ = sync.sync_document(doc_id, peer_id, &mut stream).await;
            }
        }
        p2p::Event::RequestDocument { id, hash, .. } => {
            match library
                .prisma_client()
                .asset_object()
                .find_unique(asset_object::UniqueWhereParam::HashEquals(hash.clone()))
                .exec()
                .await
                .unwrap()
            {
                Some(_asset_object_data) => {
                    // 说明有这个文件资源
                    // 不需要索要了，todo用fs查一下
                    tracing::debug!("有这个文件资源");
                    if let Some(temp_bundle_path) = library.get_temp_dir() {
                        let temp_bundle_path = temp_bundle_path.join(Uuid::new_v4().to_string());

                        tracing::debug!("temp_bundle_path: {temp_bundle_path:?}");

                        let file_hashes = vec![hash.clone()];

                        let accept_file_path_list = vec![temp_bundle_path.clone()];

                        tracing::debug!("file_hashes: {file_hashes:?}");

                        library.generate_bundle(&file_hashes, &temp_bundle_path)?;

                        // 压缩完毕
                        let _ = node.accept_share(id.clone(), accept_file_path_list).await;
                        // 发送事件
                    }
                }
                None => {
                    // 没有这个资源
                    let node = node.clone();
                    let _ = node.reject_share(id.clone()).await;
                }
            }
        }
        _ => {}
    };
    Ok(())
}
