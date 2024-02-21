pub(self) mod audio;
pub mod embedding;
pub mod index;
pub mod search;
pub mod video;

// TODO constants should be extracted into global config
pub const EMBEDDING_DIM: usize = 512;
