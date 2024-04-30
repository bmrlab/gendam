use crate::ShareInfo;
use futures::AsyncWriteExt;
use p2p::{str_to_peer_id, Node};
use p2p_block::{message::Message, SyncRequest};
use std::sync::Arc;

pub async fn handle_sync_message(
    event: sync_lib::event::Event,
    node: Arc<Node<ShareInfo>>,
) -> Result<(), anyhow::Error> {
    match event {
        sync_lib::event::Event::NeedSync { peer_id, doc_id } => {
            // 需要更新的文档
            tracing::info!("doc_id: {:?} need sync", doc_id);
            // 对方
            let peer = str_to_peer_id(peer_id.clone()).unwrap();
            // 打开流
            let mut stream = node.open_stream(peer).await.expect("open stream fail");
            // 告诉对方需要同步,把同步消息传过来
            let message =
                Message::<SyncRequest>::SyncRequest(SyncRequest { peer_id, doc_id }).to_bytes();
            let _ = stream
                .write_all(&message)
                .await
                .map_err(|error| tracing::error!("send sync request error: {:?}", error));
            tracing::debug!("发送同步请求过去");
        }
        _ => {}
    };
    Ok(())
}
