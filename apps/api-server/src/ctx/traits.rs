use crate::{
    ai::AIHandler,
    download::{DownloadReporter, DownloadStatus},
    task_queue::TaskPayload,
};
use async_trait::async_trait;
use content_library::{Library, QdrantServerInfo};
use file_handler::video::{VideoHandler, VideoTaskType};
use std::{
    boxed::Box,
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
pub trait CtxWithLibrary: Sync {
    fn is_busy(&self) -> Arc<Mutex<std::sync::atomic::AtomicBool>>;

    fn get_local_data_root(&self) -> PathBuf;
    fn get_resources_dir(&self) -> PathBuf;

    async fn load_library(&self, library_id: &str) -> Result<Library, rspc::Error>;
    async fn unload_library(&self) -> Result<(), rspc::Error>;

    fn library_id_in_store(&self) -> Option<String>;

    fn library(&self) -> Result<Library, rspc::Error>;
    fn task_tx(&self) -> Result<Sender<TaskPayload<VideoHandler, VideoTaskType>>, rspc::Error>;
    fn ai_handler(&self) -> Result<AIHandler, rspc::Error>;
    fn ai_handler_mutex(&self) -> Arc<Mutex<Option<AIHandler>>>;
    fn download_reporter(&self) -> Result<DownloadReporter, rspc::Error>;
    fn download_status(&self) -> Result<Vec<DownloadStatus>, rspc::Error>;

    fn qdrant_info(&self) -> Result<QdrantServerInfo, rspc::Error>;

    async fn trigger_unfinished_tasks(&self) -> ();
}
