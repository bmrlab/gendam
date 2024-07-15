mod constants;
mod decoder;
mod impls;
mod split;

use crate::{metadata::video::VideoMetadata, traits::FileHandler, SearchRecordType, TaskPriority};
use ai::{
    AIModel, AudioTranscriptInput, AudioTranscriptModel, AudioTranscriptOutput, ImageCaptionModel,
    MultiModalEmbeddingInput, MultiModalEmbeddingModel, MultiModalEmbeddingOutput,
    TextEmbeddingInput, TextEmbeddingModel, TextEmbeddingOutput,
};
use anyhow::bail;
use async_trait::async_trait;
pub use constants::*;
pub use impls::audio;
use qdrant_client::{
    client::QdrantClient,
    qdrant::{points_selector::PointsSelectorOneOf, Condition, Filter, PointsSelector},
};
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
    str::FromStr,
    sync::{Arc, Mutex},
};
use storage_macro::Storage;
use strum_macros::{EnumIter, EnumString};

/// Video Handler
///
/// VideoHandler is a helper to extract video artifacts and embeddings, and save results into databases.
/// ```
#[derive(Clone, Storage)]
pub struct VideoHandler {
    video_path: std::path::PathBuf,
    file_identifier: String,
    artifacts_dir: std::path::PathBuf,
    qdrant_client: Option<Arc<qdrant_client::client::QdrantClient>>,
    language_collection_name: Option<String>,
    vision_collection_name: Option<String>,
    multi_modal_embedding: Option<(
        AIModel<MultiModalEmbeddingInput, MultiModalEmbeddingOutput>,
        String,
    )>,
    image_caption: Option<(ImageCaptionModel, String)>,
    audio_transcript: Option<(AIModel<AudioTranscriptInput, AudioTranscriptOutput>, String)>,
    text_embedding: Option<(AIModel<TextEmbeddingInput, TextEmbeddingOutput>, String)>,
    metadata: Arc<Mutex<Option<VideoMetadata>>>,
}

#[derive(Clone, Debug, EnumIter, EnumString, PartialEq, Eq, Hash, strum_macros::Display)]
#[strum(serialize_all = "kebab-case")]
pub enum VideoTaskType {
    Frame,
    FrameCaption,
    FrameContentEmbedding,
    FrameCaptionEmbedding,
    Audio,
    Transcript,
    TranscriptEmbedding,
}

impl VideoTaskType {
    fn get_parent_task(&self) -> Vec<VideoTaskType> {
        match self {
            VideoTaskType::Frame => vec![],
            VideoTaskType::FrameCaption => vec![VideoTaskType::Frame],
            VideoTaskType::FrameContentEmbedding => vec![VideoTaskType::Frame],
            VideoTaskType::FrameCaptionEmbedding => vec![VideoTaskType::FrameCaption],
            VideoTaskType::Audio => vec![],
            VideoTaskType::Transcript => vec![VideoTaskType::Audio],
            VideoTaskType::TranscriptEmbedding => vec![VideoTaskType::Transcript],
        }
    }

    fn get_child_task(&self) -> Vec<VideoTaskType> {
        match self {
            VideoTaskType::Frame => vec![VideoTaskType::FrameCaption],
            VideoTaskType::FrameCaption => vec![VideoTaskType::FrameCaptionEmbedding],
            VideoTaskType::FrameContentEmbedding => vec![],
            VideoTaskType::FrameCaptionEmbedding => vec![],
            VideoTaskType::Audio => vec![VideoTaskType::Transcript],
            VideoTaskType::Transcript => vec![VideoTaskType::TranscriptEmbedding],
            VideoTaskType::TranscriptEmbedding => vec![],
        }
    }

    fn get_all_child_tasks(&self) -> Vec<VideoTaskType> {
        let mut results = HashSet::new();
        results.extend(self.get_child_task());
        for task in self.get_child_task() {
            results.extend(task.get_all_child_tasks());
        }
        return results.into_iter().collect::<Vec<_>>();
    }
}

impl VideoHandler {
    /// Create a new VideoHandler
    ///
    /// # Arguments
    ///
    /// * `video_path` - The path of the video file
    /// * `video_file_hash` - The hash of the video file
    /// * `artifacts_dir` - The path of the artifacts directory
    /// * `qdrant_client` - The qdrant client
    pub fn new(
        video_path: impl AsRef<Path>,
        video_file_hash: &str,
        artifacts_dir: impl AsRef<Path>,
        qdrant_client: Option<Arc<qdrant_client::client::QdrantClient>>,
    ) -> anyhow::Result<Self> {
        Ok(Self {
            // TODO: 暂时先使用绝对路径给 ffmpeg 使用，后续需要将文件加载到内存中传递给 ffmpeg
            video_path: video_path.as_ref().to_path_buf(),
            file_identifier: video_file_hash.to_string(),
            artifacts_dir: artifacts_dir.as_ref().to_path_buf(),
            qdrant_client,
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

    pub fn artifacts_dir(&self) -> &PathBuf {
        &self.artifacts_dir
    }

    pub fn with_multi_modal_embedding(
        self,
        multi_modal_embedding: &MultiModalEmbeddingModel,
        multi_modal_model_name: &str,
        collection_name: &str,
    ) -> Self {
        Self {
            multi_modal_embedding: Some((
                multi_modal_embedding.clone(),
                multi_modal_model_name.into(),
            )),
            vision_collection_name: Some(collection_name.to_string()),
            ..self
        }
    }

    pub fn with_image_caption(
        self,
        image_caption: &ImageCaptionModel,
        image_caption_model_name: &str,
    ) -> Self {
        Self {
            image_caption: Some((image_caption.clone(), image_caption_model_name.into())),
            ..self
        }
    }

    pub fn with_audio_transcript(
        self,
        audio_transcript: &AudioTranscriptModel,
        audio_transcript_model_name: &str,
    ) -> Self {
        Self {
            audio_transcript: Some((audio_transcript.clone(), audio_transcript_model_name.into())),
            ..self
        }
    }

    pub fn with_text_embedding(
        self,
        text_embedding: &TextEmbeddingModel,
        text_embedding_model_name: &str,
        collection_name: &str,
    ) -> Self {
        Self {
            text_embedding: Some((text_embedding.clone(), text_embedding_model_name.into())),
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

    fn multi_modal_embedding(&self) -> anyhow::Result<(&MultiModalEmbeddingModel, &str)> {
        match self.multi_modal_embedding.as_ref() {
            Some(v) => Ok((&v.0, &v.1)),
            _ => {
                bail!("multi_modal_embedding is not enabled")
            }
        }
    }

    fn image_caption(&self) -> anyhow::Result<(&ImageCaptionModel, &str)> {
        match self.image_caption.as_ref() {
            Some(v) => Ok((&v.0, &v.1)),
            _ => {
                bail!("image_caption is not enabled")
            }
        }
    }

    fn audio_transcript(&self) -> anyhow::Result<(&AudioTranscriptModel, &str)> {
        match self.audio_transcript.as_ref() {
            Some(v) => Ok((&v.0, &v.1)),
            _ => {
                bail!("audio_transcript is not enabled")
            }
        }
    }

    fn text_embedding(&self) -> anyhow::Result<(&TextEmbeddingModel, &str)> {
        match self.text_embedding.as_ref() {
            Some(v) => Ok((&v.0, &v.1)),
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

    fn qdrant_client(&self) -> anyhow::Result<Arc<QdrantClient>> {
        match self.qdrant_client.as_ref() {
            Some(v) => Ok(v.clone()),
            _ => {
                bail!("qdrant_client is not enabled")
            }
        }
    }

    pub fn get_metadata(&self) -> anyhow::Result<VideoMetadata> {
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

    pub fn save_video_segment(
        &self,
        verbose_file_name: &str,
        output_dir: impl AsRef<std::path::Path>,
        milliseconds_from: u32,
        milliseconds_to: u32,
    ) -> anyhow::Result<()> {
        let video_decoder = decoder::VideoDecoder::new(&self.video_path)?;
        video_decoder.save_video_segment(
            verbose_file_name,
            output_dir,
            milliseconds_from,
            milliseconds_to,
        )
    }

    pub async fn get_video_duration(&self) -> anyhow::Result<f64> {
        let video_decoder = decoder::VideoDecoder::new(&self.video_path)?;
        video_decoder.get_video_duration().await
    }

    pub async fn check_video_audio(&self) -> anyhow::Result<(bool, bool)> {
        let video_decoder = decoder::VideoDecoder::new(&self.video_path)?;
        video_decoder.check_video_audio().await
    }

    pub async fn generate_ts(&self, ts_index: u32, cache_dir: PathBuf) -> anyhow::Result<Vec<u8>> {
        let video_decoder = decoder::VideoDecoder::new(&self.video_path)?;
        let ts_folder = cache_dir.join("ts").join(self.file_identifier.clone());

        // 创建ts_folder
        tokio::fs::create_dir_all(ts_folder.clone()).await?;

        Ok(video_decoder.generate_ts(ts_index, ts_folder).await?)
    }
}

#[async_trait]
impl FileHandler for VideoHandler {
    async fn run_task(
        &self,
        task_type: &str,
        with_existing_artifacts: Option<bool>,
    ) -> anyhow::Result<()> {
        let task_type = VideoTaskType::from_str(task_type)?;
        let mut with_existing_artifacts = with_existing_artifacts;

        // embedding task never use existing artifacts
        if let VideoTaskType::FrameContentEmbedding
        | VideoTaskType::FrameCaptionEmbedding
        | VideoTaskType::TranscriptEmbedding = task_type
        {
            with_existing_artifacts = Some(false);
        }

        match with_existing_artifacts {
            // 如果使用存在的 artifacts，并且他们通过检查
            Some(v) if v && self.check_artifacts(&task_type) => {
                tracing::info!("run task {} with existing artifacts", task_type);
            }
            _ => {
                self.set_default_output_path(&task_type).await?;
            }
        }

        match task_type {
            VideoTaskType::Frame => self.save_frames().await?,
            VideoTaskType::FrameContentEmbedding => self.save_frame_content_embedding().await?,
            VideoTaskType::FrameCaption => self.save_frames_caption().await?,
            VideoTaskType::FrameCaptionEmbedding => self.save_frame_caption_embedding().await?,
            VideoTaskType::Audio => self.save_audio().await?,
            VideoTaskType::Transcript => self.save_transcript().await?,
            VideoTaskType::TranscriptEmbedding => self.save_transcript_embedding().await?,
            // !! DO NOT add default arm here
        }

        self.set_artifacts_result(&task_type).await?;

        Ok(())
    }

    async fn delete_artifacts_in_db(&self) -> anyhow::Result<()> {
        let qdrant = self.qdrant_client()?;
        match qdrant.list_collections().await {
            std::result::Result::Ok(collections) => {
                for collection in collections.collections.iter() {
                    qdrant
                        .delete_points(
                            &collection.name,
                            None,
                            &PointsSelector {
                                points_selector_one_of: Some(PointsSelectorOneOf::Filter(
                                    Filter::all(vec![Condition::matches(
                                        "file_identifier",
                                        self.file_identifier.to_string(),
                                    )]),
                                )),
                            },
                            None,
                        )
                        .await?;
                }
            }
            _ => {
                tracing::warn!("failed to list collections");
            }
        }

        Ok(())
    }

    async fn delete_artifacts(&self) -> anyhow::Result<()> {
        self.delete_artifacts_in_db().await?;

        // delete artifacts on file system

        self.remove_dir_all(self.artifacts_dir.clone())
            .await
            .map_err(|e| {
                tracing::error!("failed to delete artifacts: {}", e);
                e
            })?;

        Ok(())
    }

    async fn delete_artifacts_in_db_by_task(&self, task_type: &str) -> anyhow::Result<()> {
        let task_type = VideoTaskType::from_str(task_type)?;

        let mut need_to_delete_in_qdrant = vec![];
        for task in task_type.get_all_child_tasks() {
            match task {
                VideoTaskType::FrameContentEmbedding => {
                    need_to_delete_in_qdrant.push(SearchRecordType::Frame);
                }
                VideoTaskType::FrameCaptionEmbedding => {
                    need_to_delete_in_qdrant.push(SearchRecordType::FrameCaption);
                }
                VideoTaskType::TranscriptEmbedding => {
                    need_to_delete_in_qdrant.push(SearchRecordType::Transcript);
                }
                _ => {}
            }
        }

        // delete result in qdrant
        if need_to_delete_in_qdrant.len() > 0 {
            let qdrant = self.qdrant_client()?;
            match qdrant.list_collections().await {
                std::result::Result::Ok(collections) => {
                    for collection in collections.collections.iter() {
                        for record_type in need_to_delete_in_qdrant.iter() {
                            let points_selector = PointsSelector {
                                points_selector_one_of: Some(PointsSelectorOneOf::Filter(
                                    Filter::all(vec![
                                        Condition::matches(
                                            "file_identifier",
                                            self.file_identifier.to_string(),
                                        ),
                                        Condition::matches("record_type", record_type.to_string()),
                                    ]),
                                )),
                            };
                            qdrant
                                .delete_points(&collection.name, None, &points_selector, None)
                                .await?;
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }

    async fn delete_artifacts_by_task(&self, task_type: &str) -> anyhow::Result<()> {
        self.delete_artifacts_in_db_by_task(task_type).await?;

        let task_type = VideoTaskType::from_str(task_type)?;
        self._delete_artifacts_by_task(&task_type).await?;

        Ok(())
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

        if let anyhow::Result::Ok(metadata) = self.get_metadata() {
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
}
