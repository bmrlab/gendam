use std::io;

use futures::io::{AsyncRead, AsyncReadExt};
use thiserror::Error;
use uuid::Uuid;

use crate::{decode, encode};

use super::BlockSize;

/// TODO
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Range {
    /// Request the entire file
    Full,
    /// Partial range
    Partial(std::ops::Range<u64>),
}

impl Range {
    // TODO: Per field and proper error handling
    pub async fn from_stream(stream: &mut (impl AsyncRead + Unpin)) -> std::io::Result<Self> {
        let mut buf = [0u8; 1];
        stream.read_exact(&mut buf).await?;
        match buf[0] {
            0 => Ok(Self::Full),
            1 => {
                let start = read_u64_le(stream).await?;
                let end = read_u64_le(stream).await?;
                Ok(Self::Partial(start..end))
            }
            _ => Err(io::Error::new(
                io::ErrorKind::Other,
                "Invalid range discriminator",
            )),
        }
    }

    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpaceblockRequests {
    pub id: Uuid,
    pub block_size: BlockSize,
    pub requests: Vec<SpaceblockRequest>,
}

#[derive(Debug, Error)]
pub enum SpaceblockRequestsError {
    #[error("SpaceblockRequestsError::Id({0:?})")]
    Id(#[from] decode::Error),
    #[error("SpaceblockRequestsError::InvalidLen({0})")]
    InvalidLen(std::io::Error),
    #[error("SpaceblockRequestsError::SpaceblockRequest({0:?})")]
    SpaceblockRequest(#[from] SpaceblockRequestError),
    #[error("SpaceblockRequestsError::BlockSize({0:?})")]
    BlockSize(std::io::Error),
}

impl SpaceblockRequests {
    pub async fn from_stream(
        stream: &mut (impl AsyncRead + Unpin),
    ) -> Result<Self, SpaceblockRequestsError> {
        let id = decode::uuid(stream)
            .await
            .map_err(SpaceblockRequestsError::Id)?;

        tracing::info!("SpaceblockRequests::from_stream id: {:?}", id);

        let block_size = BlockSize::from_stream(stream)
            .await
            .map_err(SpaceblockRequestsError::BlockSize)?;

        tracing::info!(
            "SpaceblockRequests::from_stream block_size: {:?}",
            block_size
        );

        let mut size_u8 = [0u8; 1];
        stream
            // Max of 255 files in one request
            .read_exact(&mut size_u8)
            .await
            .map_err(SpaceblockRequestsError::InvalidLen)?;

        let size = size_u8[0] as u8;

        let mut requests = Vec::new();
        for i in 0..size {
            requests.push(SpaceblockRequest::from_stream(stream).await?);
        }

        tracing::info!("SpaceblockRequests::from_stream requests: {:?}", requests);

        Ok(Self {
            id,
            block_size,
            requests,
        })
    }

    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        let Self {
            id,
            block_size,
            requests,
        } = self;
        #[allow(clippy::panic)] // TODO: Remove this panic
        assert!(
            requests.len() <= 255,
            "Can't Spacedrop more than 255 files at once!"
        );

        let mut buf = vec![];
        encode::uuid(&mut buf, id);
        buf.append(&mut block_size.to_bytes().to_vec());
        buf.push(requests.len() as u8);
        for request in requests {
            buf.extend_from_slice(&request.to_bytes());
        }
        buf
    }
}

/// TODO
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpaceblockRequest {
    // 数据库里文件名
    pub name: String,

    // 文件哈希
    pub hash: String,

    // 文件路径
    pub path: String,

    pub size: u64,
    // TODO: Include file permissions
    pub range: Range,
    // artifact zip的大小
    pub artifact_size: u64,
    // DB
}

#[derive(Debug, Error)]
pub enum SpaceblockRequestError {
    #[error("SpaceblockRequestError::Name({0})")]
    Name(decode::Error),
    #[error("SpaceblockRequestError::Hash({0})")]
    Hash(decode::Error),
    #[error("SpaceblockRequestError::Path({0})")]
    Path(decode::Error),
    #[error("SpaceblockRequestError::Size({0})")]
    Size(std::io::Error),
    #[error("SpaceblockRequestError::ArtifactSize({0})")]
    ArtifactSize(std::io::Error),
    // TODO: From outside. Probs remove?
    #[error("SpaceblockRequestError::RangeError({0:?})")]
    RangeError(io::Error),
}

impl SpaceblockRequest {
    pub async fn from_stream(
        stream: &mut (impl AsyncRead + Unpin),
    ) -> Result<Self, SpaceblockRequestError> {
        let name = decode::string(stream)
            .await
            .map_err(SpaceblockRequestError::Name)?;

        tracing::info!("SpaceblockRequest::from_stream name: {:?}", name);

        // hash
        let hash = decode::string(stream)
            .await
            .map_err(SpaceblockRequestError::Hash)?;

        tracing::info!("SpaceblockRequest::from_stream hash: {:?}", hash);

        // path
        let path = decode::string(stream)
            .await
            .map_err(SpaceblockRequestError::Path)?;

        tracing::info!("SpaceblockRequest::from_stream path: {:?}", path);

        let size = read_u64_le(stream)
            .await
            .map_err(SpaceblockRequestError::Size)?;

        tracing::info!("SpaceblockRequest::from_stream size: {:?}", size);

        let range = Range::from_stream(stream)
            .await
            .map_err(SpaceblockRequestError::Size)?;

        tracing::info!("SpaceblockRequest::from_stream range: {:?}", range);

        let artifact_size = read_u64_le(stream)
            .await
            .map_err(SpaceblockRequestError::ArtifactSize)?;

        tracing::info!(
            "SpaceblockRequest::from_stream artifact_size: {:?}",
            artifact_size
        );

        Ok(Self {
            name,
            hash,
            path,
            size,
            range,
            artifact_size,
        })
    }

    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        let Self {
            name,
            hash,
            path,
            size,
            range,
            artifact_size,
        } = self;
        let mut buf = Vec::new();

        encode::string(&mut buf, name);
        encode::string(&mut buf, hash);
        encode::string(&mut buf, path);
        buf.extend_from_slice(&self.size.to_le_bytes());
        buf.extend_from_slice(&self.range.to_bytes());
        buf.extend_from_slice(&self.artifact_size.to_le_bytes());
        buf
    }
}

async fn read_u64_le(stream: &mut (impl AsyncRead + Unpin)) -> Result<u64, io::Error> {
    let mut buf = [0; 8];
    stream.read_exact(&mut buf).await?;
    Ok(u64::from_le_bytes(buf))
}
