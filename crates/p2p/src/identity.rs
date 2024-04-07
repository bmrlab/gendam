use libp2p::PeerId;
use rand_core::OsRng;
use thiserror::Error;
use base58::FromBase58;

pub const REMOTE_IDENTITY_LEN: usize = 32;

#[derive(Debug, Error)]
#[error(transparent)]
pub enum IdentityErr {
    #[error("{0}")]
    Dalek(#[from] ed25519_dalek::ed25519::Error),
    #[error("Invalid key length")]
    InvalidKeyLength,
}

#[derive(Debug, Clone)]
pub struct Identity(pub ed25519_dalek::SigningKey);

impl Default for Identity {
    fn default() -> Self {
        Self(ed25519_dalek::SigningKey::generate(&mut OsRng))
    }
}

impl Identity {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn get_bytes(&self) -> [u8; REMOTE_IDENTITY_LEN] {
        self.0.to_bytes()
    }
}


pub fn str_to_peer_id(id: String) -> Result<PeerId, libp2p::identity::ParseError> {
    let bytes = id.from_base58().unwrap();
    PeerId::from_bytes(&bytes)
}