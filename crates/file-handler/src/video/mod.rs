pub use self::decoder::VideoMetadata;
use ai::{
    blip::BLIP, clip::CLIP, text_embedding::TextEmbedding, whisper::Whisper, yolo::YOLO,
    BatchHandler,
};
use anyhow::Ok;
pub use constants::*;
use content_library::Library;
use std::fmt::Display;
use strum_macros::{EnumDiscriminants, EnumString};

mod constants;
mod decoder;
mod split;
mod utils;

/// Video Handler
///
/// VideoHandler is a helper to extract video artifacts and get embeddings, and save results using Prisma and faiss.
///
/// All artifacts will be saved into `local_data_dir` (one of the input argument of `new` function).
///
/// And in Tauri on macOS, it should be `/Users/%USER_NAME%/Library/Application Support/%APP_IDENTIFIER%`
/// where `APP_IDENTIFIER` is configured in `tauri.conf.json`.
///
/// # Examples
///
/// ```rust
/// let video_path = "";
/// let video_file_hash = "";
/// let resources_dir = "";
/// let local_data_dir = ""
/// let library_id = "";
///
/// let library = content_library::load_library(
///     &local_data_dir.into(),
///     &resources_dir.into(),
///     library_id,
///     clip,
///     blip,
///     whisper,
/// ).await;
///
/// let video_handler = VideoHandler::new(
///     video_path,
///     video_file_hash,
///     &library,
/// ).await.unwrap();
///
/// // get video metadata
/// video_handler.get_video_metadata().await;
///
/// // CLIP, BLIP, Whisper, TextEmbedding and YOLO models should be initialized in advanced
/// // in order to implement advanced information extraction
/// // following examples shows how to use them
/// // refer to `ai` crate for initialization of these models
/// ```
#[allow(dead_code)]
#[derive(Clone)]
pub struct VideoHandler {
    video_path: std::path::PathBuf,
    file_identifier: String,
    artifacts_dir: std::path::PathBuf,
    library: Library,
    clip: Option<BatchHandler<CLIP>>,
    blip: Option<BatchHandler<BLIP>>,
    whisper: Option<BatchHandler<Whisper>>,
    text_embedding: Option<BatchHandler<TextEmbedding>>,
    yolo: Option<BatchHandler<YOLO>>,
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
            clip: None,
            blip: None,
            whisper: None,
            text_embedding: None,
            yolo: None,
        })
    }

    pub async fn run_task(&self, task_type: VideoTaskType) -> anyhow::Result<()> {
        match task_type {
            VideoTaskType::Frame => self.save_frames().await,
            VideoTaskType::FrameContentEmbedding => self.save_frame_content_embedding().await,
            VideoTaskType::FrameCaption => self.save_frames_caption().await,
            VideoTaskType::FrameCaptionEmbedding => self.save_frame_caption_embedding().await,
            VideoTaskType::Audio => self.save_audio().await,
            VideoTaskType::Transcript => self.save_transcript().await,
            VideoTaskType::TranscriptEmbedding => self.save_transcript_embedding().await,
            VideoTaskType::FrameTags => self.save_frames_tags().await,
            VideoTaskType::FrameTagsEmbedding => self.save_frame_tags_embedding().await,
        }
    }

    pub fn get_supported_task_types(&self, with_audio: Option<bool>) -> Vec<VideoTaskType> {
        let mut task_types = vec![VideoTaskType::Frame];

        if self.clip.is_some() {
            task_types.push(VideoTaskType::FrameContentEmbedding);
        }

        if self.blip.is_some() {
            task_types.push(VideoTaskType::FrameCaption);
            if self.text_embedding.is_some() {
                task_types.push(VideoTaskType::FrameCaptionEmbedding);
            }
        }

        if let Some(with_audio) = with_audio {
            if with_audio {
                task_types.push(VideoTaskType::Audio);
            }

            if self.whisper.is_some() {
                task_types.push(VideoTaskType::Transcript);
                if self.text_embedding.is_some() {
                    task_types.push(VideoTaskType::TranscriptEmbedding);
                }
            }
        }

        // make sure the order is behind audio related tasks
        if self.yolo.is_some() {
            task_types
                .extend_from_slice(&[VideoTaskType::FrameTags, VideoTaskType::FrameTagsEmbedding]);
        }

        task_types
    }

    pub fn file_identifier(&self) -> &str {
        &self.file_identifier
    }

    fn clip(&self) -> anyhow::Result<BatchHandler<CLIP>> {
        self.clip
            .clone()
            .ok_or(anyhow::anyhow!("CLIP is not enabled"))
    }

    fn blip(&self) -> anyhow::Result<BatchHandler<BLIP>> {
        self.blip
            .clone()
            .ok_or(anyhow::anyhow!("BLIP is not enabled"))
    }

    fn whisper(&self) -> anyhow::Result<BatchHandler<Whisper>> {
        self.whisper
            .clone()
            .ok_or(anyhow::anyhow!("Whisper is not enabled"))
    }

    fn text_embedding(&self) -> anyhow::Result<BatchHandler<TextEmbedding>> {
        self.text_embedding
            .clone()
            .ok_or(anyhow::anyhow!("Text Embedding is not enabled"))
    }

    fn yolo(&self) -> anyhow::Result<BatchHandler<YOLO>> {
        self.yolo
            .clone()
            .ok_or(anyhow::anyhow!("YOLO is not enabled"))
    }

    pub fn with_clip(self, clip: BatchHandler<CLIP>) -> Self {
        Self {
            clip: Some(clip),
            ..self
        }
    }

    pub fn with_blip(self, blip: BatchHandler<BLIP>) -> Self {
        Self {
            blip: Some(blip),
            ..self
        }
    }

    pub fn with_whisper(self, whisper: BatchHandler<Whisper>) -> Self {
        Self {
            whisper: Some(whisper),
            ..self
        }
    }

    pub fn with_text_embedding(self, text_embedding: BatchHandler<TextEmbedding>) -> Self {
        Self {
            text_embedding: Some(text_embedding),
            ..self
        }
    }

    pub fn with_yolo(self, yolo: BatchHandler<YOLO>) -> Self {
        Self {
            yolo: Some(yolo),
            ..self
        }
    }

    pub async fn get_video_metadata(&self) -> anyhow::Result<VideoMetadata> {
        // TODO ffmpeg-dylib not implemented
        let video_decoder = decoder::VideoDecoder::new(&self.video_path).await?;
        video_decoder.get_video_metadata().await
    }

    pub async fn save_thumbnail(&self, seconds: Option<u64>) -> anyhow::Result<()> {
        let video_decoder = decoder::VideoDecoder::new(&self.video_path).await?;
        video_decoder
            .save_video_thumbnail(&self.artifacts_dir.join(THUMBNAIL_FILE_NAME), seconds)
            .await
    }

    /// Extract key frames from video and save results
    /// - Save into disk (a folder named by `library` and `video_file_hash`)
    /// - Save into prisma `VideoFrame` model
    async fn save_frames(&self) -> anyhow::Result<()> {
        let video_path = &self.video_path;
        let frames_dir = self.artifacts_dir.join(FRAME_DIR);

        #[cfg(feature = "ffmpeg-binary")]
        {
            let video_decoder = decoder::VideoDecoder::new(video_path).await?;
            video_decoder.save_video_frames(frames_dir.clone()).await?;
        }

        #[cfg(feature = "ffmpeg-dylib")]
        {
            let video_decoder = decoder::VideoDecoder::new(video_path);
            video_decoder.save_video_frames(frames_dir.clone()).await?;
        }

        utils::frame::save_frames(
            self.file_identifier().into(),
            self.library.prisma_client(),
            frames_dir,
        )
        .await?;

        Ok(())
    }

    /// Extract audio from video and save results
    /// - Save into disk (a folder named by `library` and `video_file_hash`)
    async fn save_audio(&self) -> anyhow::Result<()> {
        #[cfg(feature = "ffmpeg-binary")]
        {
            let video_decoder = decoder::VideoDecoder::new(&self.video_path).await?;
            video_decoder
                .save_video_audio(self.artifacts_dir.join(AUDIO_FILE_NAME))
                .await?;
        }

        #[cfg(feature = "ffmpeg-dylib")]
        {
            let video_decoder = decoder::VideoDecoder::new(&self.video_path);
            video_decoder.save_video_audio(&self.audio_path).await?;
        }

        Ok(())
    }

    /// Convert audio of the video into text
    /// **This requires extracting audio in advance**
    ///
    /// This will also save results:
    /// - Save into disk (a folder named by `library` and `video_file_hash`)
    /// - Save into prisma `VideoTranscript` model
    async fn save_transcript(&self) -> anyhow::Result<()> {
        utils::transcript::save_transcript(
            &self.artifacts_dir,
            self.file_identifier.clone(),
            self.library.prisma_client(),
            self.whisper()?,
        )
        .await
    }

    async fn save_transcript_embedding(&self) -> anyhow::Result<()> {
        utils::transcript::save_transcript_embedding(
            self.file_identifier().into(),
            self.library.prisma_client(),
            self.artifacts_dir.join(TRANSCRIPT_FILE_NAME),
            self.text_embedding()?,
            self.library.qdrant_client(),
        )
        .await?;

        Ok(())
    }

    /// Save frame content embedding into qdrant
    async fn save_frame_content_embedding(&self) -> anyhow::Result<()> {
        utils::frame::save_frame_content_embedding(
            self.file_identifier.clone(),
            self.library.prisma_client(),
            self.artifacts_dir.join(FRAME_DIR),
            self.clip()?,
            self.library.qdrant_client(),
        )
        .await
    }

    /// Save frames' captions of video
    /// **this requires extracting frames in advance**
    ///
    /// The captions will be saved:
    /// - To disk: as `.caption` file in the same place with frame file
    /// - To prisma `VideoFrameCaption` model
    async fn save_frames_caption(&self) -> anyhow::Result<()> {
        utils::caption::save_frames_caption(
            self.file_identifier().into(),
            self.artifacts_dir.join(FRAME_DIR),
            self.blip()?,
            self.library.prisma_client(),
        )
        .await
    }

    /// Save frame caption embedding into qdrant
    /// this requires extracting frames and get captions in advance
    async fn save_frame_caption_embedding(&self) -> anyhow::Result<()> {
        utils::caption::save_frame_caption_embedding(
            self.file_identifier().into(),
            self.library.prisma_client(),
            self.artifacts_dir.join(FRAME_DIR),
            utils::caption::CaptionMethod::BLIP,
            self.text_embedding()?,
            self.library.qdrant_client(),
        )
        .await?;

        Ok(())
    }

    async fn save_frames_tags(&self) -> anyhow::Result<()> {
        utils::caption::save_frames_tags(
            self.file_identifier().into(),
            self.artifacts_dir.join(FRAME_DIR),
            self.yolo()?,
            self.library.prisma_client(),
        )
        .await
    }

    async fn save_frame_tags_embedding(&self) -> anyhow::Result<()> {
        utils::caption::save_frame_caption_embedding(
            self.file_identifier().into(),
            self.library.prisma_client(),
            self.artifacts_dir.join(FRAME_DIR),
            utils::caption::CaptionMethod::YOLO,
            self.text_embedding()?,
            self.library.qdrant_client(),
        )
        .await?;

        Ok(())
    }

    /// Split video into multiple clips
    #[allow(dead_code)]
    async fn save_video_clips(&self) -> anyhow::Result<()> {
        utils::clip::save_video_clips(
            self.file_identifier.clone(),
            Some(self.artifacts_dir.join(TRANSCRIPT_FILE_NAME)),
            self.library.prisma_client(),
            self.library.qdrant_client(),
        )
        .await
    }

    #[allow(dead_code)]
    async fn save_video_clips_summarization(&self) -> anyhow::Result<()> {
        todo!("implement video clips summarization")
        // utils::clip::get_video_clips_summarization(
        //     self.file_identifier.clone(),
        //     self.resources_dir.clone(),
        //     self.client.clone(),
        // )
        // .await
    }
}
