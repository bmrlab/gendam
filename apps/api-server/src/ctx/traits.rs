use crate::{
    ai::AIHandler,
    download::{DownloadReporter, DownloadStatus},
    file_handler::TaskPayload,
    routes::p2p::ShareInfo,
};
use async_trait::async_trait;
use content_library::{Library, QdrantServerInfo};
use p2p::Node;
use std::{
    path::PathBuf,
    sync::{mpsc::Sender, Arc, Mutex},
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

#[async_trait]
pub trait CtxWithLibrary: Sync + CtxWithP2P + CtxWithAI + CtxWithDownload {
    fn is_busy(&self) -> Arc<Mutex<std::sync::atomic::AtomicBool>>;

    fn get_local_data_root(&self) -> PathBuf;
    fn get_resources_dir(&self) -> PathBuf;
    fn get_temp_dir(&self) -> PathBuf;
    fn get_cache_dir(&self) -> PathBuf;

    async fn load_library(&mut self, library_id: &str) -> Result<Library, rspc::Error>;
    async fn unload_library(&self) -> Result<(), rspc::Error>;

    fn library_id_in_store(&self) -> Option<String>;

    fn library(&self) -> Result<Library, rspc::Error>;
    fn task_tx(&self) -> Result<Sender<TaskPayload>, rspc::Error>;
    fn qdrant_info(&self) -> Result<QdrantServerInfo, rspc::Error>;

    async fn add_task(&self, task: cron::Task) -> Result<(), rspc::Error>;
}

pub trait CtxWithP2P {
    fn node(&self) -> Result<Node<ShareInfo>, rspc::Error>;
}

pub trait CtxWithAI {
    fn ai_handler(&self) -> Result<AIHandler, rspc::Error>;
    fn ai_handler_mutex(&self) -> Arc<Mutex<Option<AIHandler>>>;
}

pub trait CtxWithDownload {
    fn download_reporter(&self) -> Result<DownloadReporter, rspc::Error>;
    fn download_status(&self) -> Result<Vec<DownloadStatus>, rspc::Error>;
}