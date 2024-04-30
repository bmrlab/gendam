use libp2p::PeerId;
use p2p_block::TransferFile;
use serde::Serialize;
use tokio::sync::broadcast;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum Event<T: core::fmt::Debug> {
    ShareRequest {
        id: Uuid,
        peer_id: PeerId,
        peer_name: String,
        file_list: Vec<TransferFile>,
        share_info: T,
    },
    ShareProgress {
        id: Uuid,
        percent: u8,
        // file_path: String,
        share_info: T,
    },
    ShareTimedOut {
        id: Uuid,
    },
    ShareRejected {
        id: Uuid,
    },
}

#[derive(Debug)]
pub struct Events<T: Clone + core::fmt::Debug> {
    events: (broadcast::Sender<Event<T>>, broadcast::Receiver<Event<T>>),
}

impl<T: Clone + core::fmt::Debug> Events<T> {
    pub fn new() -> Self {
        Self {
            events: broadcast::channel::<Event<T>>(15),
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<Event<T>> {
        self.events.0.subscribe()
    }

    #[allow(clippy::result_large_err)]
    pub fn send(&self, event: Event<T>) -> Result<usize, broadcast::error::SendError<Event<T>>> {
        self.events.0.send(event)
    }
}
