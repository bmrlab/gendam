use crate::event::Event;
use crate::sync::Sync;
use automerge_repo::DocumentId;
use std::str::FromStr;
use std::sync::Arc;

pub fn spawn(sync: Arc<Sync>) -> () {
    tokio::spawn(async move {
        while let Ok(message) = sync.events.subscribe().recv().await {
            match message {
                Event::DocumentChanged { doc_id, peer_id } => {
                    tracing::info!("Received other peer document changed: {:?}", doc_id);
                    let doc_id = DocumentId::from_str(&doc_id).expect("Failed to parse doc_id");
                    // 查询有没有这个文档
                    match sync.has_document(doc_id.clone()).await {
                        Ok(doc_option) => {
                            match doc_option {
                                true => {
                                    // 有需要同步的文档
                                    tracing::debug!("有需要同步的文档");
                                    // 这里要清空 pre message
                                    sync.save_prev_has_message(
                                        doc_id.clone(),
                                        peer_id.clone(),
                                        false,
                                    )
                                    .unwrap();

                                    sync.events
                                        .send(Event::NeedSync {
                                            doc_id: doc_id.clone().as_uuid_str(),
                                            peer_id: peer_id.clone(),
                                        })
                                        .ok();
                                }
                                false => {
                                    // 没有文档需要同步
                                    tracing::debug!("没有文档需要同步");
                                }
                            }
                        }
                        Err(e) => {
                            tracing::error!("Failed to load document: {:?}", e);
                        }
                    }
                }
                _ => {}
            }
        }
    });
}
