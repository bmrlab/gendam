pub mod task_queue;
pub mod routes;
pub mod router;

use std::{
    path::PathBuf,
    sync::Arc,
};
use tokio::sync::broadcast;
// use rspc::Rspc;
use content_library::Library;
use task_queue::TaskPayload;
use vector_db::FaissIndex;

// #[derive(Clone)]
// pub struct Ctx {
//     pub local_data_root: PathBuf,
//     pub resources_dir: PathBuf,
//     // pub library: Library,
// }

pub trait CtxWithLibrary {
    fn get_local_data_root(&self) -> PathBuf;
    fn get_resources_dir(&self) -> PathBuf;
    fn load_library(&self) -> Library;
    fn get_task_tx(&self) -> Arc<broadcast::Sender<TaskPayload>>;
    fn get_index(&self) -> FaissIndex;
}

// pub const R: Rspc<Ctx> = Rspc::new();
