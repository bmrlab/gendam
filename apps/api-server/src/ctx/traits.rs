use crate::{
    ai::AIHandler,
    download::{DownloadReporter, DownloadStatus},
    routes::p2p::info::ShareInfo,
};
use async_trait::async_trait;
use content_base::ContentBase;
use content_library::Library;
use p2p::Node;
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

#[derive(Debug)]
pub struct StoreError(pub String);

pub trait CtxStore {
    fn load(&mut self) -> Result<(), StoreError>;
    fn save(&self) -> Result<(), StoreError>;
    fn insert(&mut self, key: &str, value: &str) -> Result<(), StoreError>;
    fn get(&self, key: &str) -> Option<String>;
    fn delete(&mut self, key: &str) -> Result<(), StoreError>;
}

#[derive(Debug)]
pub enum CtxError {
    BadRequest(String),
    Internal(String),
    Conflict(String),
}

impl From<CtxError> for rspc::Error {
    fn from(value: CtxError) -> Self {
        match value {
            CtxError::BadRequest(msg) => rspc::Error::new(rspc::ErrorCode::BadRequest, msg),
            CtxError::Internal(msg) => rspc::Error::new(rspc::ErrorCode::InternalServerError, msg),
            // FIXME should use 429 too many requests error code
            CtxError::Conflict(msg) => rspc::Error::new(rspc::ErrorCode::Conflict, msg),
        }
    }
}

impl From<CtxError> for anyhow::Error {
    fn from(value: CtxError) -> Self {
        match value {
            CtxError::BadRequest(msg) => anyhow::anyhow!("Bad Request: {}", msg),
            CtxError::Internal(msg) => anyhow::anyhow!("Internal Error: {}", msg),
            CtxError::Conflict(msg) => anyhow::anyhow!("Conflict: {}", msg),
        }
    }
}

#[async_trait]
pub trait CtxWithLibrary: Sync + CtxWithP2P + CtxWithAI + CtxWithDownload {
    fn is_busy(&self) -> Arc<Mutex<std::sync::atomic::AtomicBool>>;

    fn get_local_data_root(&self) -> PathBuf;
    fn get_resources_dir(&self) -> PathBuf;
    fn get_temp_dir(&self) -> PathBuf;
    fn get_cache_dir(&self) -> PathBuf;

    async fn load_library(&self, library_id: &str) -> Result<Library, CtxError>;
    async fn unload_library(&self) -> Result<(), CtxError>;

    fn library_id_in_store(&self) -> Option<String>;

    fn library(&self) -> Result<Library, CtxError>;
    fn content_base(&self) -> Result<ContentBase, CtxError>;
}

pub trait CtxWithP2P {
    fn node(&self) -> Result<Node<ShareInfo>, CtxError>;
}

pub trait CtxWithAI {
    fn ai_handler(&self) -> Result<AIHandler, CtxError>;
    fn ai_handler_mutex(&self) -> Arc<Mutex<Option<AIHandler>>>;
}

pub trait CtxWithDownload {
    fn download_reporter(&self) -> Result<DownloadReporter, CtxError>;
    fn download_status(&self) -> Result<Vec<DownloadStatus>, CtxError>;
}
