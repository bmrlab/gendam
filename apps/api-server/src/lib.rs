pub mod task_queue;
pub mod routes;
pub mod router;

use std::{
    path::PathBuf,
    pin::Pin,
    boxed::Box,
    sync::Arc
};
use tokio::sync::broadcast;
use content_library::Library;
use task_queue::TaskPayload;

// #[derive(Clone)]
// pub struct Ctx {
//     pub local_data_root: PathBuf,
//     pub resources_dir: PathBuf,
//     // pub library: Library,
// }

pub trait CtxWithLibrary {
    fn get_local_data_root(&self) -> PathBuf;
    fn get_resources_dir(&self) -> PathBuf;

    fn switch_current_library<'async_trait>(&'async_trait self, library_id: &'async_trait str)
        -> Pin<Box<dyn std::future::Future<Output = ()> + Send + 'async_trait>>
    where
        Self: Sync + 'async_trait;

    fn library(&self) -> Result<Library, rspc::Error>;

    fn get_task_tx(&self) -> Arc<broadcast::Sender<TaskPayload>>;
}

// pub const R: Rspc<Ctx> = Rspc::new();
