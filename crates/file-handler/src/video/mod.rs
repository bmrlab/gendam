mod constants;
mod decoder;
mod impls;
mod split;

use crate::{metadata::video::VideoMetadata, traits::FileHandler, TaskPriority};
use ai::{
    AIModelLoader, AsAudioTranscriptModel, AsImageCaptionModel, AsMultiModalEmbeddingModel,
    AsTextEmbeddingModel, AudioTranscriptInput, AudioTranscriptOutput, ImageCaptionInput,
    ImageCaptionOutput, MultiModalEmbeddingInput, MultiModalEmbeddingOutput, TextEmbeddingInput,
    TextEmbeddingOutput,
};
use anyhow::{bail, Ok};
use async_trait::async_trait;
pub use constants::*;
use content_library::Library;
use std::{
    str::FromStr,
    sync::{Arc, Mutex},
};
use strum::IntoEnumIterator;
use strum_macros::{EnumIter, EnumString};

/// Video Handler
///
/// VideoHandler is a helper to extract video artifacts and embeddings, and save results into databases.
/// ```
#[derive(Clone)]
pub struct VideoHandler {
    video_path: std::path::PathBuf,
    file_identifier: String,
    artifacts_dir: std::path::PathBuf,
    library: Library,
    language_collection_name: Option<String>,
    vision_collection_name: Option<String>,
    multi_modal_embedding:
        Option<AIModelLoader<MultiModalEmbeddingInput, MultiModalEmbeddingOutput>>,
    image_caption: Option<AIModelLoader<ImageCaptionInput, ImageCaptionOutput>>,
    audio_transcript: Option<AIModelLoader<AudioTranscriptInput, AudioTranscriptOutput>>,
    text_embedding: Option<AIModelLoader<TextEmbeddingInput, TextEmbeddingOutput>>,
    metadata: Arc<Mutex<Option<VideoMetadata>>>,
}

#[derive(Clone, Debug, EnumIter, EnumString, PartialEq, Eq, Hash, strum_macros::Display)]
#[strum(prefix = "VideoTask")] // 增加 prefix 避免任务名称冲突，注意这里 from_str 的时候不会考虑这个 prefix
pub enum VideoTaskType {
    Frame,
    FrameCaption,
    FrameContentEmbedding,
    FrameCaptionEmbedding,
    FrameTags,
    FrameTagsEmbedding,
    Audio,
    Transcript,
    TranscriptEmbedding,
}

impl VideoHandler {
    /// Create a new VideoHandler
    ///
    /// # Arguments
    ///
    /// * `video_path` - The path to the video file
    /// * `video_file_hash` - The hash of the video file
    /// * `library` - Current library reference
    pub fn new(
        video_path: impl AsRef<std::path::Path>,
        video_file_hash: &str,
        library: &Library,
    ) -> anyhow::Result<Self> {
        let artifacts_dir = library.artifacts_dir(video_file_hash);

        Ok(Self {
            video_path: video_path.as_ref().to_owned(),
            file_identifier: video_file_hash.to_string(),
            artifacts_dir,
            library: library.clone(),
            vision_collection_name: None,
            language_collection_name: None,
            multi_modal_embedding: None,
            image_caption: None,
            audio_transcript: None,
            text_embedding: None,
            metadata: Arc::new(Mutex::new(None)),
        })
    }

    pub fn file_identifier(&self) -> &str {
        &self.file_identifier
    }

    pub fn with_multi_modal_embedding(
        self,
        multi_modal_embedding: &dyn AsMultiModalEmbeddingModel,
        collection_name: &str,
    ) -> Self {
        Self {
            multi_modal_embedding: Some(multi_modal_embedding.get_inputs_embedding_tx().into()),
            vision_collection_name: Some(collection_name.to_string()),
            ..self
        }
    }

    pub fn with_image_caption(self, image_caption: &dyn AsImageCaptionModel) -> Self {
        Self {
            image_caption: Some(image_caption.get_images_caption_tx().into()),
            ..self
        }
    }

    pub fn with_audio_transcript(self, audio_transcript: &dyn AsAudioTranscriptModel) -> Self {
        Self {
            audio_transcript: Some(audio_transcript.get_audio_transcript_tx().into()),
            ..self
        }
    }

    pub fn with_text_embedding(
        self,
        text_embedding: &dyn AsTextEmbeddingModel,
        collection_name: &str,
    ) -> Self {
        Self {
            text_embedding: Some(text_embedding.get_texts_embedding_tx().into()),
            language_collection_name: Some(collection_name.to_string()),
            ..self
        }
    }

    pub async fn save_thumbnail(&self, seconds: Option<u64>) -> anyhow::Result<()> {
        let video_decoder = decoder::VideoDecoder::new(&self.video_path)?;
        video_decoder
            .save_video_thumbnail(&self.artifacts_dir.join(THUMBNAIL_FILE_NAME), seconds)
            .await
    }

    fn multi_modal_embedding(&self) -> anyhow::Result<&dyn AsMultiModalEmbeddingModel> {
        match self.multi_modal_embedding.as_ref() {
            Some(v) => Ok(v),
            _ => {
                bail!("multi_modal_embedding is not enabled")
            }
        }
    }

    fn image_caption(&self) -> anyhow::Result<&dyn AsImageCaptionModel> {
        match self.image_caption.as_ref() {
            Some(v) => Ok(v),
            _ => {
                bail!("image_caption is not enabled")
            }
        }
    }

    fn audio_transcript(&self) -> anyhow::Result<&dyn AsAudioTranscriptModel> {
        match self.audio_transcript.as_ref() {
            Some(v) => Ok(v),
            _ => {
                bail!("audio_transcript is not enabled")
            }
        }
    }

    fn text_embedding(&self) -> anyhow::Result<&dyn AsTextEmbeddingModel> {
        match self.text_embedding.as_ref() {
            Some(v) => Ok(v),
            _ => {
                bail!("text_embedding is not enabled")
            }
        }
    }

    fn vision_collection_name(&self) -> anyhow::Result<&str> {
        match self.vision_collection_name.as_ref() {
            Some(v) => Ok(v),
            _ => {
                bail!("vision_collection_name is not enabled")
            }
        }
    }

    fn language_collection_name(&self) -> anyhow::Result<&str> {
        match self.language_collection_name.as_ref() {
            Some(v) => Ok(v),
            _ => {
                bail!("language_collection_name is not enabled")
            }
        }
    }

    pub fn inner_metadata(&self) -> anyhow::Result<VideoMetadata> {
        let mut metadata = self.metadata.lock().unwrap();

        match &*metadata {
            Some(v) => Ok(v.clone()),
            _ => {
                // TODO ffmpeg-dylib not implemented
                let video_decoder = decoder::VideoDecoder::new(&self.video_path)?;
                let data = video_decoder.get_video_metadata()?;
                *metadata = Some(data.clone());
                Ok(data)
            }
        }
    }

    async fn _run_task(&self, task_type: VideoTaskType) -> anyhow::Result<()> {
        match task_type {
            VideoTaskType::Frame => self.save_frames().await,
            VideoTaskType::FrameContentEmbedding => self.save_frame_content_embedding().await,
            VideoTaskType::FrameCaption => self.save_frames_caption().await,
            VideoTaskType::FrameCaptionEmbedding => self.save_frame_caption_embedding().await,
            VideoTaskType::Audio => self.save_audio().await,
            VideoTaskType::Transcript => self.save_transcript().await,
            VideoTaskType::TranscriptEmbedding => self.save_transcript_embedding().await,
            _ => Ok(()),
        }
    }
}

#[async_trait]
impl FileHandler for VideoHandler {
    async fn run_task(&self, task_type: &str) -> anyhow::Result<()> {
        for enum_item in VideoTaskType::iter() {
            if enum_item.to_string().as_str() == task_type {
                return self._run_task(enum_item).await;
            }
        }

        bail!("unknown task type: {}", task_type);
    }

    async fn delete_task_artifacts(&self, task_type: &str) -> anyhow::Result<()> {
        let task_type = VideoTaskType::from_str(task_type)?;

        match task_type {
            VideoTaskType::Frame => self.delete_frames().await,
            VideoTaskType::FrameContentEmbedding => self.delete_frame_content_embedding().await,
            VideoTaskType::FrameCaption => self.delete_frames_caption().await,
            VideoTaskType::FrameCaptionEmbedding => self.delete_frame_caption_embedding().await,
            VideoTaskType::Audio => self.delete_audio().await,
            VideoTaskType::Transcript => self.delete_transcript().await,
            VideoTaskType::TranscriptEmbedding => self.delete_transcript_embedding().await,
            _ => Ok(()),
        }
    }

    fn get_supported_task_types(&self) -> Vec<(String, TaskPriority)> {
        let mut task_types = vec![(VideoTaskType::Frame, TaskPriority::Normal)];

        if self.multi_modal_embedding.is_some() {
            task_types.push((VideoTaskType::FrameContentEmbedding, TaskPriority::Normal));
        }

        if self.image_caption.is_some() {
            task_types.push((VideoTaskType::FrameCaption, TaskPriority::Low));
            if self.text_embedding.is_some() {
                task_types.push((VideoTaskType::FrameCaptionEmbedding, TaskPriority::Low));
            }
        }

        if let anyhow::Result::Ok(metadata) = self.inner_metadata() {
            if metadata.audio.is_some() {
                task_types.push((VideoTaskType::Audio, TaskPriority::Normal));

                if self.audio_transcript.is_some() {
                    task_types.push((VideoTaskType::Transcript, TaskPriority::Normal));
                    if self.text_embedding.is_some() {
                        task_types.push((VideoTaskType::TranscriptEmbedding, TaskPriority::Normal));
                    }
                }
            }
        }

        task_types
            .into_iter()
            .map(|v| (v.0.to_string(), v.1))
            .collect()
    }

    async fn update_database(&self) -> anyhow::Result<()> {
        todo!()
    }
}
