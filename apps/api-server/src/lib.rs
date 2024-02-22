use std::path::PathBuf;
use rspc::Rspc;
use content_library::Library;

#[derive(Clone)]
pub struct Ctx {
    // pub x_demo_header: Option<String>,
    pub resources_dir: PathBuf,
    pub library: Library,
    // pub local_data_dir: PathBuf,
    // pub db_url: String,
}
pub const R: Rspc<Ctx> = Rspc::new();

pub mod routes;
pub mod router;
