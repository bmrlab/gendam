use crate::traits::{ImageCaptionInput, ImageCaptionOutput, Model};
use std::path::Path;

impl Model for LLaVAPhi3Mini {
    type Item = ImageCaptionInput;
    type Output = ImageCaptionOutput;

    fn batch_size_limit(&self) -> usize {
        1
    }

    async fn process(
        &mut self,
        _items: Vec<Self::Item>,
    ) -> anyhow::Result<Vec<anyhow::Result<Self::Output>>> {
        let results: Vec<Result<String, anyhow::Error>> = vec![];
        Ok(results)
    }
}

pub struct LLaVAPhi3Mini {}

impl LLaVAPhi3Mini {
    pub async fn new(
        _device: &str,
        _gguf_model_path: impl AsRef<Path>,
        _mmproj_gguf_model_path: impl AsRef<Path>,
        _tokenizer_path: impl AsRef<Path>,
    ) -> anyhow::Result<Self> {
        Ok(Self {})
    }
}
