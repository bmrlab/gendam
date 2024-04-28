use libp2p::PeerId;
use serde::Serialize;
use tokio::sync::broadcast;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum Event {
    ShareRequest {
        id: Uuid,
        peer_id: PeerId,
        peer_name: String,
        files: Vec<ShareRequestFile>,
    },
    ShareProgress {
        id: Uuid,
        percent: u8,
        files: Vec<String>
    },
    ShareTimedOut {
        id: Uuid,
    },
    ShareRejected {
        id: Uuid,
    },
}

#[derive(Debug, Clone, Serialize)]
pub struct ShareRequestFile {
    pub name: String,
    pub hash: String,
    pub path: String,
}

#[derive(Debug)]
pub struct Events {
    events: (broadcast::Sender<Event>, broadcast::Receiver<Event>),
}

impl Events {
    pub fn new() -> Self {
        Self {
            events: broadcast::channel::<Event>(15),
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<Event> {
        self.events.0.subscribe()
    }

    #[allow(clippy::result_large_err)]
    pub fn send(&self, event: Event) -> Result<usize, broadcast::error::SendError<Event>> {
        self.events.0.send(event)
    }
}
