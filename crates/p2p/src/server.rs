use std::{
    collections::HashMap,
    net::Ipv4Addr,
    sync::{Arc, RwLock},
};

use libp2p::{
    futures::{select, StreamExt},
    identity::Keypair,
    multiaddr::Protocol,
    swarm::SwarmEvent,
    Multiaddr, PeerId,
};

use crate::{build_swarm, peer::Peer};

pub struct AutonatServer {
    pub peer_id: PeerId,
    pub identity: Keypair,
    pub port: u16,

    // 记录其他的peer
    pub peers: RwLock<HashMap<PeerId, Arc<Peer>>>,

    // 记录 metadata name operating_system(os) device_model version
    pub metadata: RwLock<HashMap<String, String>>,
}

impl AutonatServer {
    pub fn new(port: u16) -> Self {
        let bytes = "public_local_dam__autonat_server".as_bytes().to_vec();
        let identity = Keypair::ed25519_from_bytes(bytes).unwrap();
        let peer_id = identity.public().to_peer_id();
        let mut metadata = HashMap::new();
        metadata.insert("name".to_string(), "autonat_server".to_string());
        metadata.insert("operating_system".to_string(), "Linux".to_string());
        metadata.insert("version".to_string(), "0.1.0".to_string());

        Self {
            identity,
            peer_id,
            port,
            peers: Default::default(),
            metadata: RwLock::new(metadata),
        }
    }

    pub fn metadata(&self) -> HashMap<String, String> {
        self.metadata.read().unwrap().clone()
    }

    pub async fn run(&self) -> Result<(), anyhow::Error> {
        let mut swarm = build_swarm(self.identity.clone(), self.metadata()).unwrap();
        tracing::info!("server peer id: {:?}", swarm.local_peer_id());
        swarm.listen_on(
            Multiaddr::empty()
                .with(Protocol::Ip4(Ipv4Addr::UNSPECIFIED))
                .with(Protocol::Tcp(self.port)),
        )?;

        loop {
            select! {
                event = swarm.select_next_some() => match event {
                    SwarmEvent::NewListenAddr { address, .. } => tracing::info!("Listening on {address:?}"),
                    SwarmEvent::Behaviour(event) => tracing::info!("{event:?}"),
                    e => tracing::info!("{e:?}"),
                }
            }
        }
    }
}
