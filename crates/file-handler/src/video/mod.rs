mod constants;
mod decoder;
mod impls;
mod split;

pub use self::decoder::VideoMetadata;
use crate::traits::FileHandler;
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
    fmt::Display,
    sync::{Arc, Mutex},
};
use strum_macros::{EnumDiscriminants, EnumString};

/// Video Handler
///
/// VideoHandler is a helper to extract video artifacts and embeddings, and save results into databases.
/// ```
#[allow(dead_code)]
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

#[derive(Clone, Debug, EnumDiscriminants, EnumString, PartialEq, Eq, Hash)]
#[strum_discriminants(derive(strum_macros::Display))]
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

impl Display for VideoTaskType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
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
}

#[async_trait]
impl FileHandler<VideoTaskType, VideoMetadata> for VideoHandler {
    async fn run_task(&self, task_type: &VideoTaskType) -> anyhow::Result<()> {
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

    async fn delete_task_artifacts(&self, task_type: &VideoTaskType) -> anyhow::Result<()> {
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

    fn get_supported_task_types(&self) -> Vec<VideoTaskType> {
        let mut task_types = vec![VideoTaskType::Frame];

        if self.multi_modal_embedding.is_some() {
            task_types.push(VideoTaskType::FrameContentEmbedding);
        }

        if self.image_caption.is_some() {
            task_types.push(VideoTaskType::FrameCaption);
            if self.text_embedding.is_some() {
                task_types.push(VideoTaskType::FrameCaptionEmbedding);
            }
        }

        if let anyhow::Result::Ok(metadata) = self.metadata() {
            if metadata.audio.is_some() {
                task_types.push(VideoTaskType::Audio);

                if self.audio_transcript.is_some() {
                    task_types.push(VideoTaskType::Transcript);
                    if self.text_embedding.is_some() {
                        task_types.push(VideoTaskType::TranscriptEmbedding);
                    }
                }
            }
        }

        task_types
    }

    async fn update_database(&self) -> anyhow::Result<()> {
        todo!()
    }

    fn metadata(&self) -> anyhow::Result<VideoMetadata> {
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
}
