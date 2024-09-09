use super::AIModel;
use crate::ImageEmbeddingModel;
use crate::TextEmbeddingModel;
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub enum MultiModalEmbeddingInput {
    Image(PathBuf),
    Text(String),
}

pub type MultiModalEmbeddingOutput = Vec<f32>;
pub type MultiModalEmbeddingModel = AIModel<MultiModalEmbeddingInput, MultiModalEmbeddingOutput>;

impl Into<TextEmbeddingModel> for &MultiModalEmbeddingModel {
    fn into(self) -> TextEmbeddingModel {
        self.create_reference(
            |v| async { Ok(MultiModalEmbeddingInput::Text(v)) },
            |v| async { Ok(v) },
        )
    }
}

impl Into<ImageEmbeddingModel> for &MultiModalEmbeddingModel {
    fn into(self) -> ImageEmbeddingModel {
        self.create_reference(
            |v| async { Ok(MultiModalEmbeddingInput::Image(v)) },
            |v| async { Ok(v) },
        )
    }
}
