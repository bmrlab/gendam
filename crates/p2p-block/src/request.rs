use crate::{decode, encode, message, BlockSize};
use core::hash;
use futures::io::{AsyncRead, AsyncReadExt};
use futures::Future;
use serde::Serialize;
use std::io;
use std::{fmt::Display, path::PathBuf};
use thiserror::Error;
use uuid::Uuid;

pub trait StreamData: Clone + Sync + Send {
    fn from_stream(
        stream: &mut (impl AsyncRead + Unpin + Send),
    ) -> impl Future<Output = std::io::Result<Self>> + Send;
    fn to_bytes(&self) -> Vec<u8>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Range {
    /// Request the entire file
    Full,
    /// Partial range
    Partial(std::ops::Range<u64>),
}

impl StreamData for Range {
    // TODO: Per field and proper error handling
    async fn from_stream(stream: &mut (impl AsyncRead + Unpin + Send)) -> std::io::Result<Self> {
        let mut buf = [0u8; 1];
        stream.read_exact(&mut buf).await?;
        match buf[0] {
            0 => Ok(Self::Full),
            1 => {
                let start = u64::from_stream(stream).await?;
                let end = u64::from_stream(stream).await?;
                Ok(Self::Partial(start..end))
            }
            _ => Err(io::Error::new(
                io::ErrorKind::Other,
                "Invalid range discriminator",
            )),
        }
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();

        match self {
            Self::Full => buf.push(0),
            Self::Partial(range) => {
                buf.push(1);
                buf.extend_from_slice(&range.start.to_le_bytes());
                buf.extend_from_slice(&range.end.to_le_bytes());
            }
        }
        buf
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TransferFile {
    pub path: PathBuf,
    pub size: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransferRequest<T: StreamData> {
    pub id: Uuid,
    pub block_size: BlockSize,
    pub file_list: Vec<TransferFile>,
    pub info: T,
}

impl StreamData for TransferFile {
    async fn from_stream(stream: &mut (impl AsyncRead + Unpin + Send)) -> std::io::Result<Self> {
        let path = PathBuf::from_stream(stream).await?;
        let size = u64::from_stream(stream).await?;
        Ok(Self { path, size })
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.append(&mut self.path.to_bytes());
        buf.append(&mut self.size.to_le_bytes().to_vec());
        buf
    }
}

impl<T: StreamData> StreamData for TransferRequest<T> {
    // TODO complement error handle
    async fn from_stream(stream: &mut (impl AsyncRead + Unpin + Send)) -> std::io::Result<Self> {
        let id = decode::uuid(stream)
            .await
            .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
        let block_size = BlockSize::from_stream(stream).await?;
        // let size = u64::from_stream(stream).await?;
        let file_list = Vec::<TransferFile>::from_stream(stream).await?;
        let info = T::from_stream(stream).await?;

        Ok(Self {
            id,
            block_size,
            file_list,
            info,
        })
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut buf = vec![];

        encode::uuid(&mut buf, &self.id);
        buf.append(&mut self.block_size.to_bytes().to_vec());
        buf.append(&mut self.file_list.to_bytes());
        buf.append(&mut self.info.to_bytes());

        buf
    }
}

impl<T: StreamData> StreamData for Vec<T> {
    async fn from_stream(stream: &mut (impl AsyncRead + Unpin + Send)) -> std::io::Result<Self> {
        let mut buf = [0u8; 8];
        stream.read_exact(&mut buf).await?;
        let len = u64::from_le_bytes(buf) as usize;
        let mut vec = Vec::with_capacity(len);
        for _ in 0..len {
            vec.push(T::from_stream(stream).await?);
        }
        Ok(vec)
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&self.len().to_le_bytes());
        for item in self {
            buf.append(&mut item.to_bytes());
        }
        buf
    }
}

impl StreamData for PathBuf {
    async fn from_stream(stream: &mut (impl AsyncRead + Unpin + Send)) -> std::io::Result<Self> {
        let data = decode::string(stream)
            .await
            .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
        Ok(PathBuf::from(data))
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut buf = vec![];
        encode::string(&mut buf, &self.to_string_lossy());
        buf
    }
}

impl StreamData for u64 {
    async fn from_stream(stream: &mut (impl AsyncRead + Unpin + Send)) -> std::io::Result<Self> {
        let mut buf = [0; 8];
        stream.read_exact(&mut buf).await?;
        Ok(u64::from_le_bytes(buf))
    }
    fn to_bytes(&self) -> Vec<u8> {
        self.to_le_bytes().to_vec()
    }
}

// TODO 现在直接把usize当作u64处理了
impl StreamData for usize {
    async fn from_stream(stream: &mut (impl AsyncRead + Unpin + Send)) -> std::io::Result<Self> {
        let mut buf = [0; 8];
        stream.read_exact(&mut buf).await?;
        Ok(u64::from_le_bytes(buf) as usize)
    }
    fn to_bytes(&self) -> Vec<u8> {
        let v = *self as u64;
        v.to_le_bytes().to_vec()
    }
}

impl StreamData for String {
    async fn from_stream(stream: &mut (impl AsyncRead + Unpin + Send)) -> std::io::Result<Self> {
        let data = decode::string(stream).await.unwrap();
        Ok(data)
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut buf = vec![];
        encode::string(&mut buf, self);
        buf
    }
}

impl StreamData for bool {
    async fn from_stream(stream: &mut (impl AsyncRead + Unpin + Send)) -> std::io::Result<Self> {
        let mut buf = [0; 1];
        stream.read_exact(&mut buf).await?;
        Ok(buf[0] == 1)
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut buf = vec![];
        buf.push(*self as u8);
        buf
    }
}

impl<T: StreamData> StreamData for Option<T> {
    async fn from_stream(stream: &mut (impl AsyncRead + Unpin + Send)) -> std::io::Result<Self> {
        let mut buf = [0u8; 1];
        stream.read_exact(&mut buf).await?;

        if buf[0] == 0 {
            Ok(None)
        } else {
            let value = T::from_stream(stream).await?;
            Ok(Some(value))
        }
    }

    fn to_bytes(&self) -> Vec<u8> {
        match self {
            None => vec![0],
            Some(value) => {
                let mut buf = vec![1];
                buf.extend_from_slice(&value.to_bytes());
                buf
            }
        }
    }
}

impl StreamData for automerge::sync::Message {
    async fn from_stream(stream: &mut (impl AsyncRead + Unpin + Send)) -> std::io::Result<Self> {
        let mut data = Vec::new();
        stream.read_to_end(&mut data).await?;
        let message = automerge::sync::Message::decode(&data).unwrap();
        Ok(message)
    }

    fn to_bytes(&self) -> Vec<u8> {
        self.clone().encode()
    }
}

// 同步消息
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SyncMessage {
    pub peer_id: String,
    pub doc_id: String,
    pub message: Option<automerge::sync::Message>,
}

impl StreamData for SyncMessage {
    async fn from_stream(stream: &mut (impl AsyncRead + Unpin + Send)) -> std::io::Result<Self> {
        let peer_id = decode::string(stream).await.unwrap();
        let doc_id = decode::string(stream).await.unwrap();
        let message = Option::<automerge::sync::Message>::from_stream(stream)
            .await
            .unwrap();
        Ok(SyncMessage {
            peer_id,
            doc_id,
            message,
        })
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut buf = vec![];
        encode::string(&mut buf, &self.peer_id);
        encode::string(&mut buf, &self.doc_id);
        buf.extend_from_slice(&self.message.to_bytes());
        buf
    }
}

// 同步消息
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SyncRequest {
    pub peer_id: String,
    pub doc_id: String,
}

impl StreamData for SyncRequest {
    async fn from_stream(stream: &mut (impl AsyncRead + Unpin + Send)) -> std::io::Result<Self> {
        let peer_id = decode::string(stream).await.unwrap();
        let doc_id = decode::string(stream).await.unwrap();
        let mut data = Vec::new();
        stream.read_to_end(&mut data).await?;

        Ok(SyncRequest { peer_id, doc_id })
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut buf = vec![];
        encode::string(&mut buf, &self.peer_id);
        encode::string(&mut buf, &self.doc_id);
        buf
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RequestDocument {
    pub id: Uuid,
    pub hash: String,
}

impl StreamData for RequestDocument {
    async fn from_stream(stream: &mut (impl AsyncRead + Unpin + Send)) -> std::io::Result<Self> {
        let id = decode::uuid(stream).await.unwrap();
        let hash = decode::string(stream).await.unwrap();
        Ok(Self { id, hash })
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut buf = vec![];
        encode::uuid(&mut buf, &self.id);
        encode::string(&mut buf, &self.hash);
        buf
    }
}
