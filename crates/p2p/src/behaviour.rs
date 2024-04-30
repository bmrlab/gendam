use std::collections::HashMap;

use p2p_metadata;

use libp2p::{
    dcutr, gossipsub, identity,
    kad::{self, store::MemoryStore},
    mdns, ping, relay,
    swarm::NetworkBehaviour,
};
use tokio::io;

use crate::constant::PROTOCOL_VERSION;
use libp2p_stream as stream;

#[derive(NetworkBehaviour)]
pub struct Behaviour {
    pub(crate) ping: ping::Behaviour,

    /// 内网发现
    pub(crate) mdns: mdns::tokio::Behaviour,

    /// DHT可以让P2P网络中的每个节点都能够充当路由器和服务器,实现分布式定位和查询服务。每个加入网络的客户端都会在DHT网络中构建和维护其路由表,这样就可以实现全网范围内其他客户端的高效定位和发现。
    pub(crate) kademlia: kad::Behaviour<MemoryStore>, //分布式散列表

    /// Autonat 是一种 NAT traversal 技术,即穿透 NAT 以实现内外主机直接连接的技术。
    // pub(crate) auto_nat: autonat::Behaviour, // autonat 穿透 NAT 以实现内外主机直接连接的技术。

    /// 查询peerd的 metadata
    pub(crate) metadata: p2p_metadata::Behaviour,

    /// 文件分享
    pub(crate) block: stream::Behaviour,

    dcutr: dcutr::Behaviour,
    relay_client: relay::client::Behaviour,

    // 广播
    pub(crate) gossipsub: gossipsub::Behaviour,
}

impl Behaviour {
    pub fn new(
        key: identity::Keypair,
        local_public_key: identity::PublicKey,
        relay_behaviour: relay::client::Behaviour,
        metadata: HashMap<String, String>,
    ) -> Self {
        let peer_id = local_public_key.to_peer_id();
        // 必须要开启， panic 应该可以
        let mdns =
            mdns::tokio::Behaviour::new(mdns::Config::default(), peer_id).expect("open mdns fail");
        // let auto_nat = autonat::Behaviour::new(
        //     local_public_key.to_peer_id(),
        //     autonat::Config {
        //         retry_interval: Duration::from_secs(10),
        //         refresh_interval: Duration::from_secs(30),
        //         boot_delay: Duration::from_secs(5),
        //         throttle_server_period: Duration::ZERO,
        //         only_global_ips: false,
        //         ..Default::default()
        //     },
        // );

        let name: Option<String> = match metadata.get("name") {
            Some(name) => Some(name.clone()),
            None => None,
        };

        let operating_system: Option<String> = match metadata.get("operating_system") {
            Some(operating_system) => Some(operating_system.clone()),
            None => None,
        };

        let device_model: Option<String> = match metadata.get("device_model") {
            Some(device_model) => Some(device_model.clone()),
            None => None,
        };
        let version: Option<String> = match metadata.get("version") {
            Some(version) => Some(version.clone()),
            None => None,
        };

        let metadata = p2p_metadata::Behaviour::new(
            p2p_metadata::Config::new(PROTOCOL_VERSION.into(), local_public_key.clone())
                .with_metadata(name, operating_system, device_model, version),
        );

        // 广播
        let gossipsub_config = gossipsub::ConfigBuilder::default()
            .max_transmit_size(262144)
            .build()
            .map_err(|msg| io::Error::new(io::ErrorKind::Other, msg))
            .expect("fail build gossipsub config");

        let gossipsub = gossipsub::Behaviour::new(
            gossipsub::MessageAuthenticity::Signed(key.clone()),
            gossipsub_config,
        )
        .expect("Valid configuration");

        Self {
            mdns,
            metadata,
            ping: Default::default(),
            block: Default::default(),
            relay_client: relay_behaviour,
            dcutr: dcutr::Behaviour::new(peer_id),
            kademlia: kad::Behaviour::new(peer_id, MemoryStore::new(peer_id)),
            // auto_nat,
            gossipsub,
        }
    }
}
