use libp2p::StreamProtocol;

pub const BLOCK_PROTOCOL: StreamProtocol = StreamProtocol::new("/block");

pub const PROTOCOL_VERSION: &str = "/ipfs/id/1.0.0";