use std::{collections::HashMap, time::Duration};

use libp2p::{identity::Keypair, noise, tcp, yamux, Swarm};

use crate::{error::P2PError, Behaviour};

pub fn build_swarm(
    identity: Keypair,
    metadata: HashMap<String, String>,
) -> Result<Swarm<Behaviour>, P2PError<()>> {
    Ok(libp2p::SwarmBuilder::with_existing_identity(identity)
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        // .with_quic()
        .with_relay_client(noise::Config::new, yamux::Config::default)?
        .with_behaviour(|key, relay_behaviour| {
            Behaviour::new(key.public(), relay_behaviour, metadata)
        })?
        .with_swarm_config(|cfg| cfg.with_idle_connection_timeout(Duration::from_secs(u64::MAX)))
        .build())
}
