use std::{
    collections::HashMap,
    fmt::Debug,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex, PoisonError, RwLock, RwLockReadGuard,
    },
};

use anyhow::Result;
use libp2p::multiaddr::Protocol;
use libp2p::swarm::dial_opts::DialOpts;
use libp2p::{
    // autonat::{self, InboundProbeEvent, OutboundProbeEvent},
    core::Multiaddr,
    identity::Keypair,
    PeerId,
    Stream,
};

use libp2p_stream::Control;

use serde_json::json;

use tokio::sync::{mpsc, oneshot};
use uuid::Uuid;

use crate::{
    constant::BLOCK_PROTOCOL, error::P2PError, event_loop::FilePath,
    metadata::get_hardware_model_name, Events, HardwareModel,
};

use crate::event_loop::EventLoop;
use crate::peer::NetworkType;
use crate::{build_swarm, peer::Peer};

#[derive(Clone)]
pub struct Node {
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
    pub events: Arc<Events>,

    pub relay_channel: Arc<Mutex<mpsc::Sender<Multiaddr>>>,

    // 分享请求的id和配对的信道
    pub spacedrop_pairing_reqs: Arc<Mutex<HashMap<Uuid, oneshot::Sender<Option<Vec<FilePath>>>>>>,

    // 分享请求的 id和 是否已取消
    pub spacedrop_cancellations: Arc<Mutex<HashMap<Uuid, Arc<AtomicBool>>>>,
}

impl Debug for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // todo control 的debug
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

impl Node {
    pub fn new() -> Result<Self, P2PError<()>> {
        let identity = Keypair::generate_ed25519();
        let peer_id = identity.public().to_peer_id();
        tracing::debug!("local identity: {identity:#?}");
        tracing::info!("local peer_id: {peer_id:#?}");

        let metadata = Self::init_metadata();
        let swarm = build_swarm(identity.clone(), metadata.clone())?;

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
            spacedrop_pairing_reqs: Default::default(),
            spacedrop_cancellations: Default::default(),
        };

        let node_clone = node.clone();
        tokio::spawn(async move {
            let mut event_loop = EventLoop::new(Arc::new(node_clone), swarm, rx);
            match event_loop.spawn().await {
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

    // 接受文件请求
    pub async fn accept_share(
        &self,
        id: Uuid,
        file_paths: Vec<FilePath>,
    ) -> Result<(), P2PError<()>> {
        tracing::debug!("accept_share id: {id}");
        if let Some(sender) = self
            .spacedrop_pairing_reqs
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .remove(&id)
        {
            sender
                .send(Some(file_paths))
                .map_err(|err| {
                    tracing::warn!("error accepting Spacedrop '{id:?}': '{err:?}'");
                })
                .ok();
        }
        Ok(())
    }

    // 拒绝文件分享请求
    pub async fn reject_share(&self, id: Uuid) -> Result<(), P2PError<()>> {
        if let Some(sender) = self
            .spacedrop_pairing_reqs
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

    // 取消文件分享
    pub async fn cancel_share(&self, id: Uuid) -> Result<(), P2PError<()>> {
        if let Some(cancelled) = self
            .spacedrop_cancellations
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .remove(&id)
        {
            cancelled.store(true, Ordering::Relaxed);
        }
        Ok(())
    }
}
