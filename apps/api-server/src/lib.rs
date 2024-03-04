use std::path::PathBuf;
// use rspc::Rspc;
use content_library::Library;

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
}

// pub const R: Rspc<Ctx> = Rspc::new();

pub mod routes;
pub mod router;
