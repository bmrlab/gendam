use std::path::PathBuf;
use rspc::Rspc;
use content_library::Library;

#[derive(Clone)]
pub struct Ctx {
    pub local_data_root: PathBuf,
    pub resources_dir: PathBuf,
    pub library: Library,
}
pub const R: Rspc<Ctx> = Rspc::new();

pub mod routes;
pub mod router;
