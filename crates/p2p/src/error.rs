use std::{convert::Infallible, sync::PoisonError};

use libp2p::{identity::DecodingError, multiaddr, swarm::DialError, TransportError};
use libp2p_stream::OpenStreamError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum P2PError<TErr = ()> {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serde error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("Sender error: {0}")]
    Sender(#[from] tokio::sync::mpsc::error::SendError<TErr>),

    #[error("Noise error: {0}")]
    Noise(#[from] libp2p::noise::Error),

    #[error("Infallible error: {0}")]
    Infallible(#[from] Infallible),

    #[error("Decoding error: {0}")]
    DecodingError(#[from] DecodingError),

    #[error("Transport error: {0}")]
    TransportError(#[from] TransportError<TErr>),

    #[error("Multiaddr error: {0}")]
    MultiaddrError(#[from] multiaddr::Error),

    #[error("AlreadyRegistered error: {0}")]
    AlreadyRegistered(#[from] libp2p_stream::AlreadyRegistered),

    #[error("Dial error: {0}")]
    DialError(#[from] DialError),

    #[error("OpenStream error: UnsupportedProtocol")]
    OpenStreamError,

    #[error("Poison error: {0}")]
    PoisonError(#[from] PoisonError<TErr>),

    #[error("PeerNotFound error")]
    PeerNotFound,    
    
    #[error("NoRelayAddress error")]
    NoRelayAddress,

    #[error("P2P O error")]
    Other,
}

impl From<OpenStreamError> for P2PError<()> {
    fn from(err: OpenStreamError) -> Self {
        // todo 怎么写
        match err {
            OpenStreamError::UnsupportedProtocol(_) => P2PError::OpenStreamError,
            OpenStreamError::Io(e) => P2PError::Io(e),
            _ => P2PError::Other,
        }
    }
}
