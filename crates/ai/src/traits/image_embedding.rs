use super::{AIModelLoader, AIModelTx};
use std::path::PathBuf;

pub type ImageEmbeddingInput = PathBuf;
pub type ImageEmbeddingOutput = Vec<f32>;

pub trait AsImageEmbeddingModel: Send + Sync {
    fn get_images_embedding_tx(&self) -> AIModelTx<ImageEmbeddingInput, ImageEmbeddingOutput>;
}

impl AsImageEmbeddingModel for AIModelLoader<ImageEmbeddingInput, ImageEmbeddingOutput> {
    fn get_images_embedding_tx(&self) -> AIModelTx<ImageEmbeddingInput, ImageEmbeddingOutput> {
        self.tx.clone()
    }
}
