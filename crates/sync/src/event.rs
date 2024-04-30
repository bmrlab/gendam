use p2p_block::SyncMessage;
use tokio::sync::broadcast;

#[derive(Debug, Clone)]
pub enum Event {
    // 别人的文档发生改变
    DocumentChanged { doc_id: String, peer_id: String },

    // 需要同步的文档
    NeedSync { doc_id: String, peer_id: String },

    // 同步文档数据
    Sync(SyncMessage), // todo 可能要改成泛型

    // 同步文档结束
    SyncSuccess(String),

    // 同步文档失败
    SyncFailed(String),
}

#[derive(Debug)]
pub struct Events {
    events: (broadcast::Sender<Event>, broadcast::Receiver<Event>),
}

impl Events {
    pub fn new() -> Self {
        Self {
            events: broadcast::channel::<Event>(60),
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
