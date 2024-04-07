use std::{
    collections::HashMap,
    sync::{PoisonError, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use libp2p::swarm::Stream;
use libp2p::{Multiaddr, PeerId};
use libp2p_stream::OpenStreamError;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum NetworkType {
    /// default
    /// can be accessed directly
    Intranet,
    /// can only be accessed through relay
    Nat,
}

#[derive(Debug)]
pub struct Peer {
    pub(crate) peer_id: PeerId,

    #[allow(unused)]
    pub(crate) address: Multiaddr,

    // 记录 metadata name operating_system(os) device_model
    pub(crate) metadata: RwLock<HashMap<String, String>>,

    // state
    pub(crate) state: RwLock<State>,

    /// default is Intranet
    pub(crate) network: NetworkType,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct State {
    // 连接方式
    pub(crate) connection: Option<PeerConnectionCandidate>,
    // 是否在连接中
    pub(crate) connected: bool,
    // 延迟时间
    pub(crate) latency: Latency,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Latency {
    Timeout,
    Latency(u64),
}

impl Default for Latency {
    fn default() -> Self {
        Self::Timeout
    }
}

// 连接方式
#[derive(Debug, Clone, Hash, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PeerConnectionCandidate {
    SocketAddr(Multiaddr), // mdns
    Autonat,
    HDT,
}

impl Eq for Peer {}
impl PartialEq for Peer {
    fn eq(&self, other: &Self) -> bool {
        self.peer_id == other.peer_id
    }
}

impl Peer {
    pub fn new(peer_id: PeerId, multiaddr: Multiaddr) -> Self {
        Self {
            peer_id,
            address: multiaddr,
            metadata: Default::default(),
            state: Default::default(),
            network: NetworkType::Intranet,
        }
    }

    pub fn new_with_nat(peer_id: PeerId, multiaddr: Multiaddr) -> Self {
        Peer::new(peer_id, multiaddr).set_network(NetworkType::Nat)
    }

    pub fn set_network(mut self, network: NetworkType) -> Self {
        self.network = network;
        self
    }

    pub fn metadata(&self) -> RwLockReadGuard<HashMap<String, String>> {
        self.metadata.read().unwrap_or_else(PoisonError::into_inner)
    }

    pub fn metadata_mut(&self) -> RwLockWriteGuard<HashMap<String, String>> {
        self.metadata
            .write()
            .unwrap_or_else(PoisonError::into_inner)
    }

    pub fn state(&self) -> RwLockReadGuard<State> {
        self.state.read().unwrap_or_else(PoisonError::into_inner)
    }

    pub fn update_state(
        &self,
        connection: Option<PeerConnectionCandidate>,
        connected: Option<bool>,
        latency: Option<Latency>,
    ) -> Result<(), anyhow::Error> {
        {
            let mut state = self.state.write().unwrap();

            if let Some(conn) = connection {
                state.connection = Some(conn);
            }

            if let Some(conn) = connected {
                state.connected = conn;
            }

            if let Some(lat) = latency {
                state.latency = lat;
            }
        }

        Ok(())
    }

    pub fn open_stream(&self) -> Result<Stream, OpenStreamError> {
        todo!()
    }
}
