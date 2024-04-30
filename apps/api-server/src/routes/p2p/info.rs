use std::io;

use futures::AsyncRead;
use p2p_block::{
    proto::{decode, encode},
    StreamData,
};
use serde::{Deserialize, Serialize};

use crate::sync::{Folder, Item};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShareInfo {
    pub file_count: usize,
    pub doc_id_hash_list: Vec<DocIdWithHash>,
    pub folder_doc_id_list: Vec<DocIdWithFolder>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocIdWithFolder {
    pub doc_id: String,
    pub folder: Folder,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocIdWithHash {
    pub hash: String,
    pub name: String,
    pub doc_id: String,
}

impl StreamData for ShareInfo {
    async fn from_stream(stream: &mut (impl AsyncRead + Unpin + Send)) -> std::io::Result<Self> {
        let file_count = usize::from_stream(stream).await?;
        let doc_id_hash_list = Vec::<DocIdWithHash>::from_stream(stream).await?;
        let folder_doc_id_list = Vec::<DocIdWithFolder>::from_stream(stream).await?;
        Ok(Self {
            file_count,
            doc_id_hash_list,
            folder_doc_id_list,
        })
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.file_count.to_le_bytes());
        bytes.extend_from_slice(&self.doc_id_hash_list.to_bytes());
        bytes.extend_from_slice(&self.folder_doc_id_list.to_bytes());
        bytes
    }
}

impl StreamData for DocIdWithHash {
    async fn from_stream(stream: &mut (impl AsyncRead + Unpin + Send)) -> std::io::Result<Self> {
        let hash = decode::string(stream)
            .await
            .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
        let name = decode::string(stream)
            .await
            .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
        let doc_id = decode::string(stream)
            .await
            .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
        Ok(Self { doc_id, name, hash })
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        encode::string(&mut bytes, &self.hash);
        encode::string(&mut bytes, &self.name);
        encode::string(&mut bytes, &self.doc_id);
        bytes
    }
}

impl StreamData for DocIdWithFolder {
    async fn from_stream(stream: &mut (impl AsyncRead + Unpin + Send)) -> std::io::Result<Self> {
        let doc_id = decode::string(stream)
            .await
            .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
        let folder = Folder::from_stream(stream).await?;
        Ok(Self { doc_id, folder })
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        encode::string(&mut bytes, &self.doc_id);
        bytes.extend_from_slice(&self.folder.to_bytes());
        bytes
    }
}

impl StreamData for Folder {
    async fn from_stream(stream: &mut (impl AsyncRead + Unpin + Send)) -> std::io::Result<Self> {
        let name = decode::string(stream)
            .await
            .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
        let children = Vec::<Item>::from_stream(stream).await?;
        Ok(Self { name, children })
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        encode::string(&mut bytes, &self.name);
        bytes.extend_from_slice(&self.children.to_bytes());
        bytes
    }
}

impl StreamData for Item {
    async fn from_stream(stream: &mut (impl AsyncRead + Unpin + Send)) -> std::io::Result<Self> {
        let path: String = decode::string(stream)
            .await
            .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
        let is_dir = bool::from_stream(stream).await?;
        let doc_id = Option::<String>::from_stream(stream).await?;
        let hash = Option::<String>::from_stream(stream).await?;
        Ok(Self {
            path,
            is_dir,
            doc_id,
            hash,
        })
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        encode::string(&mut bytes, &self.path);
        bytes.extend_from_slice(&self.is_dir.to_bytes());
        bytes.extend_from_slice(&self.doc_id.to_bytes());
        bytes.extend_from_slice(&self.hash.to_bytes());
        bytes
    }
}
