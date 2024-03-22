#[cfg(feature = "faiss")]
pub mod faiss;

#[cfg(feature = "faiss")]
pub use faiss::*;

#[cfg(feature = "qdrant")]
mod qdrant;

#[cfg(feature = "qdrant")]
pub use qdrant::{QdrantParams, QdrantServer};

pub const DEFAULT_COLLECTION_NAME: &str = "muse-v2-512";
pub const DEFAULT_COLLECTION_DIM: u64 = 512;
