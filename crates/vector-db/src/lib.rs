#[cfg(feature = "faiss")]
pub mod faiss;

#[cfg(feature = "faiss")]
pub use faiss::*;

#[cfg(feature = "qdrant")]
mod qdrant;

#[cfg(feature = "qdrant")]
pub use qdrant::{kill as kill_qdrant_server, QdrantParams, QdrantServer};

pub const DEFAULT_VISION_COLLECTION_NAME: &str = "muse-v2-vision-512";
pub const DEFAULT_LANGUAGE_COLLECTION_NAME: &str = "muse-v2-language-1024";
pub const DEFAULT_VISION_COLLECTION_DIM: u64 = 512;
pub const DEFAULT_LANGUAGE_COLLECTION_DIM: u64 = 1024;
