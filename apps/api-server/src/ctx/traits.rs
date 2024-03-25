use std::{
    boxed::Box, path::PathBuf, pin::Pin, sync::{Arc, Mutex}
};
use tokio::sync::broadcast;
use content_library::Library;
use crate::{ai::AIHandler, task_queue::TaskPayload};

#[derive(Debug)]
pub struct StoreError(pub String);

pub trait CtxStore {
    fn load(&mut self) -> Result<(), StoreError>;
    fn save(&self) -> Result<(), StoreError>;
    fn insert(&mut self, key: &str, value: &str) -> Result<(), StoreError>;
    fn get(&self, key: &str) -> Option<String>;
}

pub trait CtxWithLibrary {
    fn get_local_data_root(&self) -> PathBuf;
    fn get_resources_dir(&self) -> PathBuf;

    fn switch_current_library<'async_trait>(&'async_trait self, library_id: &'async_trait str)
        -> Pin<Box<dyn std::future::Future<Output = ()> + Send + 'async_trait>>
    where
        Self: Sync + 'async_trait;

    fn library(&self) -> Result<Library, rspc::Error>;

    fn get_task_tx(&self) -> Arc<Mutex<broadcast::Sender<TaskPayload>>>;
    fn get_ai_handler(&self) -> AIHandler;
}
