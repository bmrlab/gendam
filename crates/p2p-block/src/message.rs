use crate::{RequestDocument, StreamData, SyncMessage, SyncRequest, TransferRequest};
use futures::io::{AsyncRead, AsyncReadExt, Error};
use std::fmt::Display;
use uuid::Uuid;

#[derive(Debug, PartialEq)]
pub enum Message<T: StreamData> {
    Share(TransferRequest<T>),
    Sync(SyncMessage),
    SyncRequest(SyncRequest),

    // 索要数据
    RequestDocument(RequestDocument),
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
            1 => {
                let sync = SyncMessage::from_stream(stream).await.unwrap();
                Ok(Self::Sync(sync))
            }
            2 => {
                let sync_request = SyncRequest::from_stream(stream).await.unwrap();
                Ok(Self::SyncRequest(sync_request))
            }
            3 => {
                let request_document = RequestDocument::from_stream(stream).await.unwrap();
                Ok(Self::RequestDocument(request_document))
            }
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
            Self::Sync(sync) => {
                let mut bytes = vec![1];
                bytes.extend_from_slice(&sync.to_bytes());
                bytes
            }
            Self::SyncRequest(sync_request) => {
                let mut bytes = vec![2];
                bytes.extend_from_slice(&sync_request.to_bytes());
                bytes
            }
            Self::RequestDocument(RequestDocument) => {
                let mut bytes = vec![3];
                bytes.extend_from_slice(&RequestDocument.to_bytes());
                bytes
            }
        }
    }
}
