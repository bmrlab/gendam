use super::AIModel;

pub type TextEmbeddingInput = String;
pub type TextEmbeddingOutput = Vec<f32>;
pub type TextEmbeddingModel = AIModel<TextEmbeddingInput, TextEmbeddingOutput>;
