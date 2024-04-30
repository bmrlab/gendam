use crate::event_loop::EventLoop;
use crate::peer::NetworkType;
use crate::{build_swarm, peer::Peer};
use crate::{
    constant::BLOCK_PROTOCOL, error::P2PError, metadata::get_hardware_model_name, Events,
    HardwareModel,
};
use crate::{str_to_peer_id, PubsubMessage};
use anyhow::Result;
use libp2p::futures::{AsyncReadExt, AsyncWriteExt};
use libp2p::gossipsub;
use libp2p::multiaddr::Protocol;
use libp2p::swarm::dial_opts::DialOpts;
use libp2p::{core::Multiaddr, identity::Keypair, PeerId, Stream};
use libp2p_stream::Control;
use p2p_block::message::Message;
use p2p_block::{BlockSize, StreamData, Transfer, TransferFile, TransferRequest};
use serde_json::json;
use std::future::Future;
use std::time::Duration;
use std::{
    collections::HashMap,
    fmt::Debug,
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex, PoisonError, RwLock, RwLockReadGuard,
    },
};
use tokio::fs::File;
use tokio::io::BufReader;
use tokio::sync::{mpsc, oneshot};
use tokio::time::{sleep, Instant};
use uuid::Uuid;

#[derive(Clone)]
pub struct Node<T: StreamData + core::fmt::Debug> {
    pub peer_id: PeerId,
    pub identity: Keypair,

    /// The address of the relay server, if any.
    pub relay_address: Option<Multiaddr>,

    // 记录其他的peer
    pub peers: Arc<RwLock<HashMap<PeerId, Arc<Peer>>>>,

    // 记录 metadata name operating_system(os) device_model version
    pub metadata: HashMap<String, String>,

    // libp2p_stream 创建流
    pub control: Control,

    // websocket 事件
    pub events: Arc<Events<T>>,

    pub relay_channel: Arc<Mutex<mpsc::Sender<Multiaddr>>>,

    // 分享请求的 id 和配对的 channel
    pub share_pairing_reqs: Arc<Mutex<HashMap<Uuid, oneshot::Sender<Option<Vec<PathBuf>>>>>>,

    // 分享请求的 id 和对应的传输信息
    pub share_payloads: Arc<Mutex<HashMap<Uuid, TransferRequest<T>>>>,

    // 分享请求的 id 和 是否已取消
    pub share_cancellations: Arc<Mutex<HashMap<Uuid, Arc<AtomicBool>>>>,
    // broadcast
    // pub broadcast: oneshot::Sender<PubsubMessage>
}

impl<T: StreamData + core::fmt::Debug> Debug for Node<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // TODO control 的debug
        f.debug_struct("Node")
            .field("peer_id", &self.peer_id)
            .field("identity", &self.identity)
            .field("peers", &self.peers)
            .field("metadata", &self.metadata)
            .field("relay_address", &self.relay_address)
            .field("ws", &self.events)
            .finish()
    }
}

impl<T: StreamData + core::fmt::Debug + 'static> Node<T> {
    pub fn new(s: tokio::sync::mpsc::Receiver<PubsubMessage>) -> Result<Self, P2PError<()>> {
        let identity = Keypair::generate_ed25519();
        let peer_id = identity.public().to_peer_id();
        tracing::debug!("local identity: {identity:#?}");
        tracing::info!("local peer_id: {peer_id:#?}");

        let metadata = Self::init_metadata();

        let gossipsub_topic = gossipsub::IdentTopic::new("sync");

        let mut swarm = build_swarm(identity.clone(), metadata.clone())?;

        tracing::info!("Subscribing to {gossipsub_topic:?}");
        swarm
            .behaviour_mut()
            .gossipsub
            .subscribe(&gossipsub_topic)
            .map_err(|e| {
                tracing::error!("gossipsub fail subscribe: {e}");
                e
            })
            .expect("gossipsub fail subscribe");

        let control = swarm.behaviour().block.new_control();

        let (tx, rx) = mpsc::channel::<Multiaddr>(1);

        let node = Node {
            peer_id,
            identity,
            peers: Default::default(),
            metadata,
            control,
            events: Arc::new(Events::new()),
            relay_channel: Arc::new(Mutex::new(tx)),
            relay_address: None,
            share_pairing_reqs: Default::default(),
            share_payloads: Default::default(),
            share_cancellations: Default::default(),
        };

        let node_clone = node.clone();
        tokio::spawn(async move {
            let mut event_loop = EventLoop::new(Arc::new(node_clone), swarm, rx, gossipsub_topic);
            match event_loop.spawn(s).await {
                Ok(_) => {}
                Err(e) => {
                    tracing::error!("event loop error: {e}");
                }
            }
        });
        Ok(node)
    }

    fn init_metadata() -> HashMap<String, String> {
        let mut node_metadata = HashMap::new();
        node_metadata.insert(
            "name".to_string(),
            whoami::fallible::hostname().unwrap_or_else(|_| "unknown".to_string()),
        );

        node_metadata.insert(
            "operating_system".to_string(),
            whoami::platform().to_string(),
        );
        whoami::devicename_os();
        node_metadata.insert(
            "device_model".to_string(),
            get_hardware_model_name()
                .unwrap_or(HardwareModel::Other)
                .to_string(),
        );
        node_metadata.insert("version".to_string(), env!("CARGO_PKG_VERSION").to_string());
        node_metadata
    }

    /// call dial(get_peer_dial_opts(peer_id))
    /// local peer can access directly in kademlia
    /// peer behind nat can access through relay
    pub fn get_peer_dial_opts(&self, peer_id: PeerId) -> Result<impl Into<DialOpts>, P2PError> {
        let peers_guard = self.peers.read();
        let peer_option = match &peers_guard {
            Ok(p) => p.get(&peer_id),
            Err(_) => None,
        };

        match peer_option {
            Some(peer) => match peer.network {
                NetworkType::Intranet => Ok(DialOpts::from(peer_id)),
                NetworkType::Nat => match self.relay_address.as_ref() {
                    Some(relay_address) => Ok(relay_address
                        .clone()
                        .with(Protocol::P2pCircuit)
                        .with(Protocol::P2p(peer_id))
                        .into()),
                    None => Err(P2PError::NoRelayAddress),
                },
            },
            None => Err(P2PError::PeerNotFound),
        }
    }

    pub async fn state(
        &self,
    ) -> Result<serde_json::Value, P2PError<RwLockReadGuard<'_, HashMap<String, String>>>> {
        let metadata = self.metadata();
        let peers = self.get_peers().clone();
        // tracing::debug!("state peers: {:#?}", peers);
        Ok(json!({
            "peer_id": self.peer_id.to_base58(),
            "metadata": json!({
                "name": metadata.get("name"),
                "operating_system": metadata.get("operating_system"),
                "device_model": metadata.get("device_model"),
                "version": metadata.get("version"),
            }),
            "peers": peers.iter().map(|(peer_id, p)| json!({
                "peer_id": peer_id.to_base58(),
                "metadata": p.metadata().clone(),
                "state": p.state().clone()
            })).collect::<Vec<_>>()
        }))
    }

    pub fn get_peers(&self) -> RwLockReadGuard<HashMap<PeerId, Arc<Peer>>> {
        self.peers.read().unwrap_or_else(PoisonError::into_inner)
    }

    pub fn metadata(&self) -> HashMap<String, String> {
        self.metadata.clone()
    }

    pub fn hostname(&self) -> String {
        self.metadata
            .get("name")
            .unwrap_or(&"unknown".to_string())
            .to_string()
    }

    // 开启一个流
    pub async fn open_stream(&self, peer_id: PeerId) -> Result<Stream, P2PError> {
        // todo 判断peer 在peers里
        let mut control = self.control.clone();

        // 开启新的 stream
        let stream = control.open_stream(peer_id, BLOCK_PROTOCOL).await?; // todo 这里的error 还有问题

        Ok(stream)
    }

    pub async fn add_relay_address(&mut self, address: Multiaddr) -> Result<(), P2PError> {
        self.relay_address = Some(address.clone());
        self.relay_channel
            .lock()
            .map_err(|_| P2PError::PoisonError(PoisonError::new(())))?
            .send(address)
            .await
            .map_err(|_| P2PError::Sender(mpsc::error::SendError(())))
    }

    pub async fn start_share<TFn, TFut>(
        &self,
        peer_id: &str,
        file_path_list: Vec<PathBuf>,
        share_info: T,
        on_finished: TFn,
    ) -> Result<Uuid, P2PError<()>>
    where
        TFn: FnOnce() -> TFut + Send + 'static,
        TFut: Future<Output = ()> + Send + 'static,
    {
        let peer_id = str_to_peer_id(peer_id.into()).map_err(|_| P2PError::PeerNotFound)?;
        let peer = match self.get_peers().get(&peer_id) {
            Some(peer) => peer.clone(),
            None => {
                return Err(P2PError::PeerNotFound);
            }
        };

        tracing::debug!("starting file share with {peer:#?}");

        let mut stream = match self.open_stream(peer_id).await {
            Ok(stream) => stream,
            Err(error) => {
                return Err(error);
            }
        };

        tracing::debug!("open stream: {:#?}", stream);

        let id = Uuid::new_v4();
        let events = self.events.clone();
        let share_cancellations = self.share_cancellations.clone();

        tokio::spawn(async move {
            tracing::debug!("({id}): connected, sending message");
            let transfer_files = file_path_list
                .iter()
                .map(|v| TransferFile {
                    path: v.clone(),
                    size: std::fs::metadata(v).unwrap().len(),
                })
                .collect::<Vec<_>>();

            let requests = TransferRequest {
                id,
                block_size: BlockSize::default(),
                file_list: transfer_files.clone(),
                info: share_info.clone(),
            };

            let message = Message::Share(requests.clone());

            tracing::info!("sending message: {:#?}", message);

            let buf = &message.to_bytes();

            if let Err(err) = stream.write_all(buf).await {
                tracing::error!("({id}): failed to send header: {err}");
                return;
            }

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
                    events.send(crate::Event::ShareRejected { id }).ok();
                    return;
                }
                1 => {}       // Okay
                _ => todo!(), // TODO: Proper error
            }

            let cancelled = Arc::new(AtomicBool::new(false));
            share_cancellations
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .insert(id, cancelled.clone());

            tracing::info!("({id}): starting transfer");

            // 记录开始时间
            let i = Instant::now();

            let share_info_clone = share_info.clone();

            let mut transfer = Transfer::new(
                &requests,
                |percent| {
                    events
                        .send(crate::Event::ShareProgress {
                            id,
                            percent,
                            share_info: share_info_clone.clone(),
                        })
                        .ok();
                    tracing::info!("({id}): progress: {percent}%");
                },
                &cancelled,
            );

            for file in transfer_files {
                if let Ok(file) = File::open(&file.path).await {
                    let file = BufReader::new(file);
                    if let Err(e) = transfer.send(&mut stream, file).await {
                        tracing::error!("({id}): failed to send file: {e}");
                        // TODO error to frontend
                        // return;
                    }
                }
            }

            // 传输完成
            tracing::debug!("({id}): finished; took '{:?}", i.elapsed());

            on_finished().await;
        });

        Ok(id)
    }

    // 接受文件请求
    pub async fn accept_share(
        &self,
        id: Uuid,
        file_path_list: Vec<PathBuf>,
    ) -> Result<(), P2PError<()>> {
        tracing::debug!("accept_share id: {id}");
        if let Some(sender) = self
            .share_pairing_reqs
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .remove(&id)
        {
            // remove payload info
            self.share_payloads
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .remove(&id);

            sender
                // .send(Some(file_paths))
                .send(Some(file_path_list))
                .map_err(|err| {
                    tracing::warn!("error accepting share '{id:?}': '{err:?}'");
                })
                .ok();
        }
        Ok(())
    }

    // 拒绝文件分享请求
    pub async fn reject_share(&self, id: Uuid) -> Result<(), P2PError<()>> {
        if let Some(sender) = self
            .share_pairing_reqs
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .remove(&id)
        {
            sender
                .send(None)
                .map_err(|err| {
                    tracing::warn!("error rejecting Spacedrop '{id:?}': '{err:?}'");
                })
                .ok();
        }
        Ok(())
    }

    pub fn get_payload(&self, id: Uuid) -> Result<Option<TransferRequest<T>>, P2PError<()>> {
        if let Some(payload) = self
            .share_payloads
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .get(&id)
        {
            Ok(Some(payload.clone()))
        } else {
            Ok(None)
        }
    }

    // 取消文件分享
    pub async fn cancel_share(&self, id: Uuid) -> Result<(), P2PError<()>> {
        if let Some(cancelled) = self
            .share_cancellations
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .remove(&id)
        {
            cancelled.store(true, Ordering::Relaxed);
        }
        Ok(())
    }
}
