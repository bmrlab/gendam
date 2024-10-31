pub mod artifacts;

use ai::{
    AudioTranscriptModel, ImageCaptionModel, LLMModel, MultiModalEmbeddingModel, TextEmbeddingModel,
};
use anyhow::bail;
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};
use storage_macro::Storage;

#[derive(Clone, Storage)]
pub struct ContentBaseCtx {
    artifacts_dir: PathBuf,
    tmp_dir: PathBuf,
    multi_modal_embedding: Option<(Arc<MultiModalEmbeddingModel>, String)>,
    text_embedding: Option<(Arc<TextEmbeddingModel>, String)>,
    audio_transcript: Option<(Arc<AudioTranscriptModel>, String)>,
    image_caption: Option<(Arc<ImageCaptionModel>, String)>,
    llm: Option<(Arc<LLMModel>, String)>,
    text_tokenizer: Option<(ai::tokenizers::Tokenizer, String)>,
}

impl ContentBaseCtx {
    pub fn new(artifacts_dir: impl AsRef<Path>, tmp_dir: impl AsRef<Path>) -> Self {
        Self {
            artifacts_dir: artifacts_dir.as_ref().to_path_buf(),
            tmp_dir: tmp_dir.as_ref().to_path_buf(),
            multi_modal_embedding: None,
            text_embedding: None,
            audio_transcript: None,
            image_caption: None,
            llm: None,
            text_tokenizer: None,
        }
    }

    /// A absolute path where all tmp artifacts will be stored.
    pub fn tmp_dir(&self) -> &PathBuf {
        &self.tmp_dir
    }

    pub fn with_multi_modal_embedding(
        mut self,
        multi_modal_embedding: Arc<MultiModalEmbeddingModel>,
        model_name: &str,
    ) -> Self {
        self.multi_modal_embedding = Some((multi_modal_embedding, model_name.to_string()));
        self
    }

    pub fn with_text_embedding(
        mut self,
        text_embedding: Arc<TextEmbeddingModel>,
        model_name: &str,
    ) -> Self {
        self.text_embedding = Some((text_embedding, model_name.to_string()));
        self
    }

    pub fn with_image_caption(
        mut self,
        image_caption: Arc<ImageCaptionModel>,
        model_name: &str,
    ) -> Self {
        self.image_caption = Some((image_caption, model_name.to_string()));
        self
    }

    pub fn with_audio_transcript(
        mut self,
        audio_transcript: Arc<AudioTranscriptModel>,
        model_name: &str,
    ) -> Self {
        self.audio_transcript = Some((audio_transcript, model_name.to_string()));
        self
    }

    pub fn with_llm(mut self, llm: Arc<LLMModel>, model_name: &str) -> Self {
        self.llm = Some((llm, model_name.to_string()));
        self
    }

    /// 目前这个是专门给 audio transcript 和 raw text 的 chunking 用的
    pub fn with_text_tokenizer(
        mut self,
        tokenizer: ai::tokenizers::Tokenizer,
        tokenizer_name: &str,
    ) -> Self {
        self.text_tokenizer = Some((tokenizer, tokenizer_name.into()));
        self
    }

    pub fn multi_modal_embedding(&self) -> anyhow::Result<(&MultiModalEmbeddingModel, &str)> {
        match self.multi_modal_embedding.as_ref() {
            Some(v) => Ok((&v.0, &v.1)),
            _ => {
                bail!("multi_modal_embedding is not enabled")
            }
        }
    }

    pub fn text_embedding(&self) -> anyhow::Result<(&TextEmbeddingModel, &str)> {
        match self.text_embedding.as_ref() {
            Some(v) => Ok((&v.0, &v.1)),
            _ => {
                bail!("text_embedding is not enabled")
            }
        }
    }

    pub fn image_caption(&self) -> anyhow::Result<(&ImageCaptionModel, &str)> {
        match self.image_caption.as_ref() {
            Some(v) => Ok((&v.0, &v.1)),
            _ => {
                bail!("image_caption is not enabled")
            }
        }
    }

    pub fn audio_transcript(&self) -> anyhow::Result<(&AudioTranscriptModel, &str)> {
        match self.audio_transcript.as_ref() {
            Some(v) => Ok((&v.0, &v.1)),
            _ => {
                bail!("audio_transcript is not enabled")
            }
        }
    }

    pub fn llm(&self) -> anyhow::Result<(&LLMModel, &str)> {
        match self.llm.as_ref() {
            Some(v) => Ok((&v.0, &v.1)),
            _ => {
                bail!("llm is not enabled")
            }
        }
    }

    /// 目前这个是专门给 audio transcript 和 raw text 的 chunking 用的
    pub fn text_tokenizer(&self) -> anyhow::Result<(&ai::tokenizers::Tokenizer, &str)> {
        match self.text_tokenizer.as_ref() {
            Some(v) => Ok((&v.0, &v.1)),
            _ => {
                bail!("llm_tokenizer is not enabled")
            }
        }
    }

    /// Generate text embedding and save it to `path`.
    /// Empty string will be ignored and no error will be raised.
    pub async fn save_text_embedding(
        &self,
        text: &str,
        path: impl AsRef<Path>,
    ) -> anyhow::Result<()> {
        // generate embedding for empty string will cause strange search results, see https://github.com/bmrlab/gendam/issues/90
        if text.is_empty() {
            return Ok(());
        }

        let (text_embedding_model, _) = self.text_embedding()?;
        let model_input: ai::TextEmbeddingInput = text.to_string();
        let embedding = text_embedding_model.process_single(model_input).await?;
        self.write(
            path.as_ref().to_path_buf(),
            serde_json::to_string(&embedding)?.into(),
        )
        .await?;

        Ok(())
    }
}
