mod p2p_event;
mod sync_event;

use crate::{
    event::{p2p_event::handle_p2p_message, sync_event::handle_sync_message},
    ShareInfo,
};
use content_library::Library;
use p2p::Node;
use std::sync::Arc;

/*
    所有的事件处理
    1. p2p事件
    2. 同步事件
*/
pub async fn spawn(node: Node<ShareInfo>, library: Library) -> Result<(), anyhow::Error> {
    let sync = library.sync();
    let mut p2p_rx = node.events.subscribe();
    let mut sync_rx = sync.events.subscribe();
    let library_clone = Arc::new(library);
    tokio::spawn(async move {
        loop {
            tokio::select! {
                // p2p 事件
                Ok(event) = p2p_rx.recv() => handle_p2p_message(event, Arc::new(node.clone()), sync.clone(), library_clone.clone()).await.map_err(|error| tracing::error!("p2p event error: {:?}", error)).unwrap(),
                // 同步事件
                Ok(event) = sync_rx.recv() => handle_sync_message(event, Arc::new(node.clone())).await.map_err(|error| tracing::error!("sync event error: {:?}", error)).unwrap(),
            }
        }
    });
    Ok(())
}
