use super::AIModel;
use std::path::PathBuf;

pub type ImageEmbeddingInput = PathBuf;
pub type ImageEmbeddingOutput = Vec<f32>;
pub type ImageEmbeddingModel = AIModel<ImageEmbeddingInput, ImageEmbeddingOutput>;
