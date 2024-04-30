use std::fmt::Display;
use futures::io::{AsyncRead, AsyncReadExt, Error};
use crate::{StreamData, TransferRequest};

#[derive(Debug, PartialEq, Eq)]
pub enum Message<T: StreamData> {
    Share(TransferRequest<T>),
}

impl<T: StreamData> Message<T> {
    pub async fn from_stream(stream: &mut (impl AsyncRead + Unpin + Send)) -> Result<Self, Error> {
        let mut buf = [0u8; 1];
        stream.read_exact(&mut buf).await?;
        let discriminator = buf[0];
        match discriminator {
            0 => Ok(Self::Share(
                TransferRequest::from_stream(stream).await.unwrap(),
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
