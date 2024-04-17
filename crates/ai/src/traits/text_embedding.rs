use super::{AIModelLoader, AIModelTx};

pub type TextEmbeddingInput = String;
pub type TextEmbeddingOutput = Vec<f32>;

pub trait AsTextEmbeddingModel: Send + Sync {
    fn get_texts_embedding_tx(&self) -> AIModelTx<TextEmbeddingInput, TextEmbeddingOutput>;
}

impl AsTextEmbeddingModel for AIModelLoader<TextEmbeddingInput, TextEmbeddingOutput> {
    fn get_texts_embedding_tx(&self) -> AIModelTx<TextEmbeddingInput, TextEmbeddingOutput> {
        self.tx.clone()
    }
}
