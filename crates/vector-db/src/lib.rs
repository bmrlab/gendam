#[cfg(feature = "faiss")]
pub mod faiss;

#[cfg(feature = "faiss")]
pub use faiss::*;

#[cfg(feature = "qdrant")]
mod qdrant;

#[cfg(feature = "qdrant")]
pub use qdrant::{QdrantParams, QdrantServer};

pub const VIDEO_FRAME_INDEX_NAME: &str = "frame-embedding";
pub const VIDEO_FRAME_CAPTION_INDEX_NAME: &str = "frame-caption-embedding";
pub const VIDEO_TRANSCRIPT_INDEX_NAME: &str = "transcript-embedding";
