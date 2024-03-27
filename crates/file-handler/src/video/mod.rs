pub use self::decoder::VideoMetadata;
use ai::blip::BLIP;
use ai::clip::CLIP;
use ai::whisper::{Whisper, WhisperParams};
use ai::BatchHandler;
use anyhow::Ok;
use content_library::Library;
use prisma_lib::PrismaClient;
use qdrant_client::client::QdrantClient;
use std::fs;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;

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
/// let resources_dir = "";
/// let local_data_dir = ""
/// let library_id = "";
///
/// let library = content_library::load_library(
///     &local_data_dir.into(),
///     &resources_dir.into(),
///     library_id,
/// ).await;
///
/// let video_handler = VideoHandler::new(
///     video_path,
///     resources_dir,
///     &library,
/// ).await.unwrap();
///
/// // get video metadata
/// video_handler.get_video_metadata().await;
///
/// // get frames and save their embedding into faiss
/// video_handler.get_frames().await;
/// video_handler.get_frame_content_embedding().await;
///
/// // get audio, then extract text, and finally save text embedding in faiss
/// video_handler.get_audio().await;
/// video_handler.get_transcript().await;
/// video_handler.get_transcript_embedding().await;
/// ```
#[allow(dead_code)]
#[derive(Clone)]
pub struct VideoHandler {
    video_path: std::path::PathBuf,
    file_identifier: String,
    frames_dir: std::path::PathBuf,
    audio_path: std::path::PathBuf,
    transcript_path: std::path::PathBuf,
    library: Library,
    client: Arc<PrismaClient>,
    qdrant: Arc<QdrantClient>,
    clip: BatchHandler<CLIP>,
    blip: BatchHandler<BLIP>,
    whisper: BatchHandler<Whisper>,
}

impl VideoHandler {
    /// Create a new VideoHandler
    ///
    /// # Arguments
    ///
    /// * `video_path` - The path to the video file
    /// * `library` - Current library reference
    /// * `clip` - CLIP batch handler from ai crate
    /// * `blip` - BLIP batch handler from ai crate
    /// * `whisper` - whisper batch handler from ai crate
    pub async fn new(
        video_path: impl AsRef<std::path::Path>,
        video_file_hash: &str,
        library: &Library,
        clip: BatchHandler<CLIP>,
        blip: BatchHandler<BLIP>,
        whisper: BatchHandler<Whisper>,
    ) -> anyhow::Result<Self> {
        // let bytes = std::fs::read(&video_path)?;
        // let file_sha256 = sha256::digest(&bytes);
        let artifacts_dir = library.artifacts_dir.join(video_file_hash);
        let frames_dir = artifacts_dir.join("frames");

        fs::create_dir_all(&artifacts_dir)?;
        fs::create_dir_all(&frames_dir)?;

        Ok(Self {
            video_path: video_path.as_ref().to_owned(),
            file_identifier: video_file_hash.to_string(),
            audio_path: artifacts_dir.join("audio.wav"),
            frames_dir,
            transcript_path: artifacts_dir.join("transcript.txt"),
            library: library.clone(),
            qdrant: library.qdrant_server.get_client().clone(),
            client: library.prisma_client(),
            clip,
            blip,
            whisper,
        })
    }

    pub fn file_identifier(&self) -> &str {
        &self.file_identifier
    }

    pub async fn get_video_metadata(&self) -> anyhow::Result<VideoMetadata> {
        // TODO ffmpeg-dylib not implemented
        let video_decoder = decoder::VideoDecoder::new(&self.video_path).await?;
        video_decoder.get_video_metadata().await
    }

    /// Extract key frames from video
    /// and save the results in local data directory (in a folder named by file identifier)
    pub async fn get_frames(&self) -> anyhow::Result<()> {
        let video_path = &self.video_path;

        #[cfg(feature = "ffmpeg-binary")]
        {
            let video_decoder = decoder::VideoDecoder::new(video_path).await?;
            video_decoder.save_video_frames(&self.frames_dir).await?;
        }

        #[cfg(feature = "ffmpeg-dylib")]
        {
            let video_decoder = decoder::VideoDecoder::new(video_path);
            video_decoder.save_video_frames(&self.frames_dir).await?;
        }

        Ok(())
    }

    /// Extract audio from video
    /// and save the results in local data directory (in a folder named by file identifier)
    pub async fn get_audio(&self) -> anyhow::Result<()> {
        #[cfg(feature = "ffmpeg-binary")]
        {
            let video_decoder = decoder::VideoDecoder::new(&self.video_path).await?;
            video_decoder.save_video_audio(&self.audio_path).await?;
        }

        #[cfg(feature = "ffmpeg-dylib")]
        {
            let video_decoder = decoder::VideoDecoder::new(&self.video_path);
            video_decoder.save_video_audio(&self.audio_path).await?;
        }

        Ok(())
    }

    /// Convert audio of the video into text
    /// this requires extracting audio in advance
    ///
    /// And the transcript will be saved in the same directory with audio
    pub async fn get_transcript(&self) -> anyhow::Result<()> {
        let result = self
            .whisper
            .process_single((
                self.audio_path.clone(),
                Some(WhisperParams {
                    enable_translate: false,
                    ..Default::default()
                }),
            ))
            .await?;

        // write results into json file
        let mut file = tokio::fs::File::create(&self.transcript_path).await?;
        let json = serde_json::to_string(&result.items())?;
        file.write_all(json.as_bytes()).await?;

        utils::transcript::save_transcript(
            result,
            self.file_identifier.clone(),
            self.client.clone(),
        ).await?;

        Ok(())
    }

    #[deprecated(note = "deprecated for now, output language of transcript is not stable for now")]
    /// Get transcript embedding
    /// this requires extracting transcript in advance
    pub async fn get_transcript_embedding(&self) -> anyhow::Result<()> {
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

    /// Get frame content embedding
    pub async fn get_frame_content_embedding(&self) -> anyhow::Result<()> {
        utils::frame::get_frame_content_embedding(
            self.file_identifier.clone(),
            self.client.clone(),
            &self.frames_dir,
            self.clip.clone(),
            self.qdrant.clone(),
        )
        .await?;

        Ok(())
    }

    /// Get frames' captions of video
    /// this requires extracting frames in advance
    ///
    /// All caption will be saved in .caption file in the same place with frame file
    pub async fn get_frames_caption(&self) -> anyhow::Result<()> {
        utils::caption::get_frames_caption(&self.frames_dir, self.blip.clone()).await
    }

    /// Get frame caption embedding
    /// this requires extracting frames and get captions in advance
    pub async fn get_frame_caption_embedding(&self) -> anyhow::Result<()> {
        utils::caption::get_frame_caption_embedding(
            self.file_identifier().into(),
            self.client.clone(),
            &self.frames_dir,
            self.clip.clone(),
            self.qdrant.clone(),
        )
        .await?;

        Ok(())
    }

    /// Split video into multiple clips
    pub async fn get_video_clips(&self) -> anyhow::Result<()> {
        utils::clip::get_video_clips(
            self.file_identifier.clone(),
            Some(&self.transcript_path),
            self.client.clone(),
            self.qdrant.clone(),
        )
        .await
    }

    pub async fn get_video_clips_summarization(&self) -> anyhow::Result<()> {
        todo!("implement video clips summarization")
        // utils::clip::get_video_clips_summarization(
        //     self.file_identifier.clone(),
        //     self.resources_dir.clone(),
        //     self.client.clone(),
        // )
        // .await
    }
}

#[test_log::test(tokio::test)]
async fn test_handle_video() {
    // let video_path = "/Users/zhuo/Desktop/file_v2_f566a493-ad1b-4324-b16f-0a4c6a65666g 2.MP4";
    // let resources_dir = "/Users/zhuo/Library/Application Support/cc.musedam.local/resources";
    // let local_data_dir =
    //     std::path::Path::new("/Users/zhuo/Library/Application Support/cc.musedam.local")
    //         .to_path_buf();
    // let library = content_library::load_library(
    //     &local_data_dir,
    //     "78a978d85b8ff26cc202aa6d244ed576ef5a187873c49255d3980df69deedb8a",
    // )
    // .await
    // .unwrap();

    // let video_handler = VideoHandler::new(video_path, &library).await;

    // if video_handler.is_err() {
    //     tracing::error!("failed to create video handler");
    // }
    // let video_handler = video_handler.unwrap();

    // tracing::info!("file handler initialized");

    // let metadata = video_handler.get_video_metadata().await;
    // tracing::info!("got video metadata: {:?}", metadata);

    // video_handler
    //     .get_frames()
    //     .await
    //     .expect("failed to get frames");

    // tracing::info!("got frames");

    // video_handler
    //     .get_frame_content_embedding()
    //     .await
    //     .expect("failed to get frame content embedding");

    // tracing::debug!("got frame content embedding");

    // video_handler
    //     .indexes
    //     .frame_index
    //     .flush()
    //     .await
    //     .expect("failed to flush index");

    // video_handler
    //     .get_audio()
    //     .await
    //     .expect("failed to get audio");

    // tracing::info!("got audio");

    // video_handler
    //     .get_transcript()
    //     .await
    //     .expect("failed to get transcript");

    // tracing::info!("got transcript");

    // video_handler
    //     .get_transcript_embedding()
    //     .await
    //     .expect("failed to get transcript embedding");

    // video_handler
    //     .indexes
    //     .transcript_index
    //     .flush()
    //     .await
    //     .expect("failed to flush index");

    // video_handler
    //     .get_frames_caption()
    //     .await
    //     .expect("failed to get frames caption");
    // video_handler
    //     .get_frame_caption_embedding()
    //     .await
    //     .expect("failed to get frame caption embedding");

    // video_handler
    //     .indexes
    //     .flush()
    //     .await
    //     .expect("failed to flush index");

    // video_handler
    //     .get_video_clips()
    //     .await
    //     .expect("failed to get video clips");

    // video_handler
    //     .get_video_clips_summarization()
    //     .await
    //     .expect("failed to get video clips summarization");

    // tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
}
