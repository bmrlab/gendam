use anyhow::{bail, Ok};
use content_library::Library;
use prisma_lib::PrismaClient;
use qdrant_client::client::QdrantClient;
use std::fs;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::sync::RwLock;

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
/// let local_data_dir = "";
/// let resources_dir = "";
/// let db_url = "";
/// let client = prisma_lib::new_client_with_url(&db_url).await.unwrap();
/// let client = Arc::new(RwLock::new(client));
///
/// let video_handler = VideoHandler::new(
///     video_path,
///     local_data_dir,
///     resources_dir,
///     client,
/// ).await.unwrap();
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
    resources_dir: std::path::PathBuf,
    file_identifier: String,
    frames_dir: std::path::PathBuf,
    audio_path: std::path::PathBuf,
    transcript_path: std::path::PathBuf,
    library: Library,
    client: Arc<RwLock<PrismaClient>>,
    qdrant: Arc<QdrantClient>,
}

impl VideoHandler {
    /// Create a new VideoHandler
    ///
    /// # Arguments
    ///
    /// * `video_path` - The path to the video file
    /// * `local_data_dir` - The path to the local data directory, where artifacts (frame, transcript, etc.) will be saved into
    /// * `resources_dir` - The path to the resources directory (src-tauri/resources), where contains model files
    /// * `client` - The prisma client
    pub async fn new(
        video_path: impl AsRef<std::path::Path>,
        resources_dir: impl AsRef<std::path::Path>,
        library: &Library,
        client: Arc<RwLock<PrismaClient>>,
        qdrant: Arc<QdrantClient>,
    ) -> anyhow::Result<Self> {
        let bytes = std::fs::read(&video_path)?;
        let file_sha256 = sha256::digest(&bytes);
        let artifacts_dir = library.artifacts_dir.join(&file_sha256);
        let frames_dir = artifacts_dir.join("frames");

        fs::create_dir_all(&artifacts_dir)?;
        fs::create_dir_all(&frames_dir)?;

        // make sure clip files are downloaded
        let resources_dir_clone = resources_dir.as_ref().to_owned();
        let clip_handle = tokio::spawn(ai::clip::CLIP::new(
            ai::clip::model::CLIPModel::ViTB32,
            resources_dir_clone,
        ));

        let resources_dir_clone = resources_dir.as_ref().to_owned();
        let whisper_handle = tokio::spawn(ai::whisper::Whisper::new(resources_dir_clone));

        let clip_result = clip_handle.await;
        let whisper_result = whisper_handle.await;

        if let Err(clip_err) = clip_result.as_ref().unwrap() {
            bail!("Failed to initialize clip model: {clip_err}");
        }
        if let Err(whisper_err) = whisper_result.unwrap() {
            bail!("Failed to initialize whisper model: {whisper_err}");
        }

        Ok(Self {
            video_path: video_path.as_ref().to_owned(),
            resources_dir: resources_dir.as_ref().to_owned(),
            file_identifier: file_sha256.clone(),
            audio_path: artifacts_dir.join("audio.wav"),
            frames_dir,
            transcript_path: artifacts_dir.join("transcript.txt"),
            library: library.clone(),
            qdrant,
            client,
        })
    }

    pub fn file_identifier(&self) -> &str {
        &self.file_identifier
    }

    /// Extract key frames from video
    /// and save the results in local data directory (in a folder named by file identifier)
    pub async fn get_frames(&self) -> anyhow::Result<()> {
        let video_path = &self.video_path;

        #[cfg(feature = "ffmpeg-binary")]
        {
            let video_decoder = decoder::VideoDecoder::new(video_path, &self.resources_dir).await?;
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
            let video_decoder =
                decoder::VideoDecoder::new(&self.video_path, &self.resources_dir).await?;
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
        let mut whisper = ai::whisper::Whisper::new(&self.resources_dir).await?;
        let result = whisper.transcribe(&self.audio_path, None)?;

        // write results into json file
        let mut file = tokio::fs::File::create(&self.transcript_path).await?;
        let json = serde_json::to_string(&result.items())?;
        file.write_all(json.as_bytes()).await?;

        Ok(())
    }

    /// Get transcript embedding
    /// this requires extracting transcript in advance
    pub async fn get_transcript_embedding(&self) -> anyhow::Result<()> {
        let clip_model = self.get_clip_instance().await?;
        let clip_model = Arc::new(RwLock::new(clip_model));

        utils::transcript::get_transcript_embedding(
            self.file_identifier().into(),
            self.client.clone(),
            &self.transcript_path,
            clip_model,
            self.qdrant.clone(),
        )
        .await?;

        Ok(())
    }

    /// Get frame content embedding
    pub async fn get_frame_content_embedding(&self) -> anyhow::Result<()> {
        let clip_model = self.get_clip_instance().await?;
        let clip_model = Arc::new(RwLock::new(clip_model));

        utils::frame::get_frame_content_embedding(
            self.file_identifier.clone(),
            self.client.clone(),
            &self.frames_dir,
            clip_model,
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
        let blip_model = ai::blip::BLIP::new(&self.resources_dir).await?;
        let blip_model = Arc::new(RwLock::new(blip_model));

        utils::caption::get_frames_caption(&self.frames_dir, blip_model).await
    }

    /// Get frame caption embedding
    /// this requires extracting frames and get captions in advance
    pub async fn get_frame_caption_embedding(&self) -> anyhow::Result<()> {
        let clip_model = self.get_clip_instance().await?;
        let clip_model = Arc::new(RwLock::new(clip_model));

        utils::caption::get_frame_caption_embedding(
            self.file_identifier().into(),
            self.client.clone(),
            &self.frames_dir,
            clip_model,
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
            Some(&self.frames_dir),
            self.client.clone(),
        )
        .await
    }

    pub async fn get_video_clips_summarization(&self) -> anyhow::Result<()> {
        utils::clip::get_video_clips_summarization(
            self.file_identifier.clone(),
            self.resources_dir.clone(),
            self.client.clone(),
        )
        .await
    }

    async fn get_clip_instance(&self) -> anyhow::Result<ai::clip::CLIP> {
        ai::clip::CLIP::new(ai::clip::model::CLIPModel::ViTB32, &self.resources_dir).await
    }
}

#[test_log::test(tokio::test)]
async fn test_handle_video() {
    let video_path = "/Users/zhuo/Desktop/file_v2_f566a493-ad1b-4324-b16f-0a4c6a65666g 2.MP4";
    // let video_path = "/Users/zhuo/Desktop/屏幕录制2022-11-30 11.43.29.mov";
    // let resources_dir = "/Users/zhuo/dev/bmrlab/tauri-dam-test-playground/target/debug/resources";
    let resources_dir = "/Users/zhuo/Library/Application Support/cc.musedam.local/resources";
    let local_data_dir =
        std::path::Path::new("/Users/zhuo/Library/Application Support/cc.musedam.local")
            .to_path_buf();
    // let library =
    //     content_library::create_library_with_title(&local_data_dir, "dev test library").await;
    let library = content_library::load_library(
        &local_data_dir,
        "98f19afbd2dee7fa6415d5f523d36e8322521e73fd7ac21332756330e836c797",
    ).await;

    let qdrant_client = QdrantClient::from_url("http://localhost:6333")
        .build()
        .expect("failed to build qdrant client");
    let qdrant_client = Arc::new(qdrant_client);

    let video_handler =
        VideoHandler::new(video_path, resources_dir, &library,
            Arc::clone(&library.prisma_client), qdrant_client).await;

    if video_handler.is_err() {
        tracing::error!("failed to create video handler");
    }
    let video_handler = video_handler.unwrap();

    tracing::info!("file handler initialized");

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

    video_handler
        .get_video_clips_summarization()
        .await
        .expect("failed to get video clips summarization");

    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
}
