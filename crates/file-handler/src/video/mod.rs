pub use self::decoder::VideoMetadata;
use ai::{blip::BLIP, clip::CLIP, whisper::Whisper, BatchHandler};
use anyhow::Ok;
pub use constants::*;
use content_library::Library;
use std::{fmt::Display, fs};
use strum_macros::EnumDiscriminants;

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
/// // CLIP, BLIP, Whisper model should be initialized in advanced
/// // in order to implement advanced information extraction
/// // following examples shows how to use them
/// // refer to `ai` crate for initialization of these models
///
/// // save frames into disk and prisma
/// video_handler.save_frames().await;
/// // save frames embedding into qdrant
/// let video_handler = video_handler.with_clip(clip);
/// video_handler.save_frame_content_embedding().await;
///
/// // save audio into disk
/// video_handler.save_audio().await;
/// // save transcript into disk and prisma
/// let video_handler = video_handler.with_whisper(whisper);
/// video_handler.save_transcript().await;
/// // save transcript embedding into qdrant
/// let video_handler = video_handler.with_clip(clip);
/// video_handler.save_transcript_embedding().await;
///
/// // save frames' captions into disk and prisma
/// let video_handler = video_handler.with_blip(blip);
/// video_handler.save_frames_caption().await;
/// // save frames' captions embedding into qdrant
/// let video_handler = video_handler.with_whisper(whisper);
/// video_handler.save_frame_caption_embedding().await;
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
}

#[derive(Clone, Debug, EnumDiscriminants)]
#[strum_discriminants(derive(strum_macros::Display))]
pub enum VideoTaskType {
    Frame,
    FrameCaption,
    FrameContentEmbedding,
    FrameCaptionEmbedding,
    Audio,
    Transcript,
    #[allow(dead_code)]
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
        let artifacts_dir = library.artifacts_dir.join(video_file_hash);
        fs::create_dir_all(&artifacts_dir)?;

        Ok(Self {
            video_path: video_path.as_ref().to_owned(),
            file_identifier: video_file_hash.to_string(),
            artifacts_dir,
            library: library.clone(),
            clip: None,
            blip: None,
            whisper: None,
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
            _ => Ok(()),
        }
    }

    pub fn get_supported_task_types(&self) -> Vec<VideoTaskType> {
        vec![
            VideoTaskType::Frame,
            VideoTaskType::FrameContentEmbedding,
            VideoTaskType::FrameCaptionEmbedding,
            VideoTaskType::FrameCaption,
            VideoTaskType::Audio,
            VideoTaskType::Transcript,
        ]
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

    pub async fn get_video_metadata(&self) -> anyhow::Result<VideoMetadata> {
        // TODO ffmpeg-dylib not implemented
        let video_decoder = decoder::VideoDecoder::new(&self.video_path).await?;
        video_decoder.get_video_metadata().await
    }

    /// Extract key frames from video and save results
    /// - Save into disk (a folder named by `library` and `video_file_hash`)
    /// - Save into prisma `VideoFrame` model
    pub async fn save_frames(&self) -> anyhow::Result<()> {
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
    pub async fn save_audio(&self) -> anyhow::Result<()> {
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
    pub async fn save_transcript(&self) -> anyhow::Result<()> {
        utils::transcript::save_transcript(
            &self.artifacts_dir,
            self.file_identifier.clone(),
            self.library.prisma_client(),
            self.whisper()?,
        )
        .await
    }

    #[deprecated(note = "deprecated for now, output language of transcript is not stable for now")]
    pub async fn save_transcript_embedding(&self) -> anyhow::Result<()> {
        // utils::transcript::get_transcript_embedding(
        //     self.file_identifier().into(),
        //     self.client.clone(),
        //     &self.transcript_path,
        //     self.clip.clone(),
        //     self.qdrant.clone(),
        // )
        // .await?;

        Ok(())
    }

    /// Save frame content embedding into qdrant
    pub async fn save_frame_content_embedding(&self) -> anyhow::Result<()> {
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
    pub async fn save_frames_caption(&self) -> anyhow::Result<()> {
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
    pub async fn save_frame_caption_embedding(&self) -> anyhow::Result<()> {
        utils::caption::save_frame_caption_embedding(
            self.file_identifier().into(),
            self.library.prisma_client(),
            self.artifacts_dir.join(FRAME_DIR),
            self.clip()?,
            self.library.qdrant_client(),
        )
        .await?;

        Ok(())
    }

    /// Split video into multiple clips
    pub async fn save_video_clips(&self) -> anyhow::Result<()> {
        utils::clip::save_video_clips(
            self.file_identifier.clone(),
            Some(self.artifacts_dir.join(TRANSCRIPT_FILE_NAME)),
            self.library.prisma_client(),
            self.library.qdrant_client(),
        )
        .await
    }

    pub async fn save_video_clips_summarization(&self) -> anyhow::Result<()> {
        todo!("implement video clips summarization")
        // utils::clip::get_video_clips_summarization(
        //     self.file_identifier.clone(),
        //     self.resources_dir.clone(),
        //     self.client.clone(),
        // )
        // .await
    }
}
