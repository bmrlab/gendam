// use tokio::io::{AsyncRead, AsyncReadExt, Error};
use futures::io::{AsyncRead, AsyncReadExt, Error};

use crate::SpaceblockRequests;

#[derive(Debug, PartialEq, Eq)]
pub enum Message {
    Share(SpaceblockRequests),
}

impl Message {
    pub async fn from_stream(stream: &mut (impl AsyncRead + Unpin)) -> Result<Self, Error> {
        let mut buf = [0u8; 1];
        stream.read_exact(&mut buf).await?;
        let discriminator = buf[0];
        match discriminator {
            0 => Ok(Self::Share(
                SpaceblockRequests::from_stream(stream).await.unwrap(),
            )),
            d => {
                todo!("error")
            }
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            Self::Share(transfer_request) => {
                let mut bytes = vec![0];
                bytes.extend_from_slice(&transfer_request.to_bytes());
                bytes
            }
        }
    }
}
