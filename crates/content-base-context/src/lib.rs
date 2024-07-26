pub mod artifacts;

use ai::{
    tokenizers::Tokenizer, AudioTranscriptModel, LLMModel, MultiModalEmbeddingModel,
    TextEmbeddingModel, TextEmbeddingOutput,
};
use anyhow::bail;
use qdrant_client::Qdrant;
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};
use storage_macro::Storage;

#[derive(Clone, Storage)]
pub struct ContentBaseCtx {
    qdrant: Arc<Qdrant>,
    vision_collection_name: String,
    language_collection_name: String,
    artifacts_dir: PathBuf,
    multi_modal_embedding: Option<(Arc<MultiModalEmbeddingModel>, String)>,
    text_embedding: Option<(Arc<TextEmbeddingModel>, String)>,
    audio_transcript: Option<(Arc<AudioTranscriptModel>, String)>,
    llm: Option<(Arc<LLMModel>, String)>,
    llm_tokenizer: Option<Tokenizer>,
}

impl ContentBaseCtx {
    pub fn new(
        qdrant: Arc<Qdrant>,
        vision_collection_name: &str,
        language_collection_name: &str,
        artifacts_dir: impl AsRef<Path>,
    ) -> Self {
        Self {
            qdrant,
            vision_collection_name: vision_collection_name.to_string(),
            language_collection_name: language_collection_name.to_string(),
            artifacts_dir: artifacts_dir.as_ref().to_path_buf(),
            multi_modal_embedding: None,
            text_embedding: None,
            audio_transcript: None,
            llm: None,
            llm_tokenizer: None,
        }
    }

    pub fn qdrant(&self) -> &Qdrant {
        self.qdrant.as_ref()
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

    pub fn with_audio_transcript(
        mut self,
        audio_transcript: Arc<AudioTranscriptModel>,
        model_name: &str,
    ) -> Self {
        self.audio_transcript = Some((audio_transcript, model_name.to_string()));
        self
    }

    pub fn with_llm(mut self, llm: Arc<LLMModel>, tokenizer: Tokenizer, model_name: &str) -> Self {
        self.llm = Some((llm, model_name.to_string()));
        self.llm_tokenizer = Some(tokenizer);
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

    pub fn llm_tokenizer(&self) -> anyhow::Result<&Tokenizer> {
        match self.llm_tokenizer.as_ref() {
            Some(v) => Ok(v),
            _ => {
                bail!("llm_tokenizer is not enabled")
            }
        }
    }

    pub fn language_collection_name(&self) -> &str {
        &self.language_collection_name
    }

    pub fn vision_collection_name(&self) -> &str {
        &self.vision_collection_name
    }

    pub async fn save_text_embedding(
        &self,
        text: &str,
        path: impl AsRef<Path>,
    ) -> anyhow::Result<TextEmbeddingOutput> {
        let text_embedding = self.text_embedding()?.0;
        let embedding = text_embedding.process_single(text.to_string()).await?;
        self.write(
            path.as_ref().to_path_buf(),
            serde_json::to_string(&embedding)?.into(),
        )
        .await?;
        Ok(embedding)
    }
}
