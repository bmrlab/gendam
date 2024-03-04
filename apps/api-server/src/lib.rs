use std::path::PathBuf;
use rspc::Rspc;
use content_library::Library;

#[derive(Clone)]
pub struct Ctx {
    pub local_data_root: PathBuf,
    pub resources_dir: PathBuf,
    pub library: Library,
}

impl Ctx {
    pub fn load(&self) -> Library {
        self.library.clone()
    }
}

pub const R: Rspc<Ctx> = Rspc::new();

pub mod routes;
pub mod router;
