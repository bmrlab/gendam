use super::index::VideoIndex;
use crate::index::EmbeddingIndex;
use ai::whisper::WhisperItem;
use anyhow::{anyhow, Ok};
use prisma_lib::{
    new_client_with_url, video_frame, video_frame_caption, video_transcript, PrismaClient,
};
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::sync::RwLock;
use tracing::{debug, error};

pub mod decoder;

/// Video Handler
///
/// VideoHandler is a helper to struct video artifacts and get embeddings.
///
/// All artifacts will be saved into `local_data_dir` (one of the input argument of `new` function).
///
/// And in Tauri on macOS, it should be `/Users/xx/Library/Application Support/%APP_IDENTIFIER%`
/// where `APP_IDENTIFIER` is configured in `tauri.conf.json`.
///
/// # Examples
///
/// ```rust
/// let video_path = "";
/// let local_data_dir = "";
/// let resources_dir = "";
/// let db_url = "";
///
/// let video_handler = VideoHandler::new(
///     video_path,
///     local_data_dir,
///     resources_dir,
///     db_url,
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
///
/// // flush embedding index into disk
/// video_handler.indexes().flush().await;
/// ```
#[derive(Clone, Debug)]
pub struct VideoHandler {
    video_path: std::path::PathBuf,
    resources_dir: std::path::PathBuf,
    file_identifier: String,
    frames_dir: std::path::PathBuf,
    audio_path: std::path::PathBuf,
    transcript_path: std::path::PathBuf,
    client: Arc<RwLock<PrismaClient>>,
    indexes: VideoIndex,
}

impl VideoHandler {
    /// Create a new VideoHandler
    ///
    /// # Arguments
    ///
    /// * `video_path` - The path to the video file
    /// * `local_data_dir` - The path to the local data directory, where artifacts (frame, transcript, etc.) will be saved into
    /// * `resources_dir` - The path to the resources directory (src-tauri/resources), where contains model files
    /// * `db_url` - The path for sqlite file
    pub async fn new(
        video_path: impl AsRef<std::path::Path>,
        local_data_dir: impl AsRef<std::path::Path>,
        resources_dir: impl AsRef<std::path::Path>,
        db_url: impl AsRef<std::path::Path>,
    ) -> anyhow::Result<Self> {
        let bytes = std::fs::read(&video_path)?;
        let file_sha256 = sha256::digest(&bytes);
        let artifacts_dir = local_data_dir.as_ref().join(&file_sha256);
        let frames_dir = artifacts_dir.join("frames");
        let index_dir = local_data_dir.as_ref().join("index");

        fs::create_dir_all(&artifacts_dir)?;
        fs::create_dir_all(&frames_dir)?;
        fs::create_dir_all(&index_dir)?;

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
            return Err(anyhow!("Failed to download clip model: {clip_err}"));
        }
        if let Err(whisper_err) = whisper_result.unwrap() {
            return Err(anyhow!("Failed to download whisper model: {whisper_err}"));
        }

        debug!("clip and whisper models downloaded");

        let indexes = VideoIndex::new(index_dir, clip_result.unwrap().unwrap().dim()).expect("Failed to create indexes");
        let client = new_client_with_url(db_url.as_ref().to_str().unwrap().as_ref()).await?;

        Ok(Self {
            video_path: video_path.as_ref().to_owned(),
            resources_dir: resources_dir.as_ref().to_owned(),
            file_identifier: file_sha256.clone(),
            audio_path: artifacts_dir.join("audio.wav"),
            frames_dir,
            transcript_path: artifacts_dir.join("transcript.txt"),
            indexes,
            client: Arc::new(RwLock::new(client)),
        })
    }

    pub fn file_identifier(&self) -> &str {
        &self.file_identifier
    }

    pub fn indexes(&self) -> &VideoIndex {
        &self.indexes
    }

    /// Extract key frames from video
    /// and save the results in local data directory (in a folder named by file identifier)
    pub async fn get_frames(&self) -> anyhow::Result<()> {
        let video_decoder = decoder::video::VideoDecoder::new(&self.video_path);
        video_decoder.save_video_frames(&self.frames_dir).await?;

        Ok(())
    }

    /// Extract audio from video
    /// and save the results in local data directory (in a folder named by file identifier)
    pub async fn get_audio(&self) -> anyhow::Result<()> {
        let video_decoder = decoder::video::VideoDecoder::new(&self.video_path);
        video_decoder.save_video_audio(&self.audio_path).await?;

        Ok(())
    }

    /// Convert audio of the video into text
    /// this requires extracting audio in advance
    ///
    /// And the transcript will be saved in the same directory with audio
    pub async fn get_transcript(&self) -> anyhow::Result<()> {
        let mut whisper = ai::whisper::Whisper::new(&self.resources_dir).await?;
        let result = whisper.transcribe(&self.audio_path)?;

        // write results into json file
        let mut file = tokio::fs::File::create(&self.transcript_path).await?;
        let json = serde_json::to_string(&result)?;
        file.write_all(json.as_bytes()).await?;

        Ok(())
    }

    /// Get transcript embedding
    /// this requires extracting transcript in advance
    pub async fn get_transcript_embedding(&self) -> anyhow::Result<()> {
        let file = File::open(&self.transcript_path)?;
        let reader = BufReader::new(file);

        // Read the JSON contents of the file as an instance of `WhisperItem`
        let whisper_results: Vec<WhisperItem> = serde_json::from_reader(reader)?;

        let clip_model = self.get_clip_instance().await?;
        let clip_model = Arc::new(RwLock::new(clip_model));

        let embedding_index = Arc::new(self.indexes.transcript_index.clone());

        let mut join_set = tokio::task::JoinSet::new();

        for item in whisper_results {
            // if item is some like [MUSIC], just skip it
            // TODO need to make sure all filter rules
            if item.text.starts_with("[") || item.text.starts_with("(") {
                continue;
            }

            let clip_model = Arc::clone(&clip_model);
            let file_identifier = self.file_identifier.clone();
            let embedding_index = Arc::clone(&embedding_index);
            let client = self.client.clone();

            join_set.spawn(async move {
                // write data using prisma
                // FIXME here use write to make sure only one thread can using prisma client
                let client = client.write().await;
                let x = client.video_transcript().upsert(
                    video_transcript::file_identifier_start_timestamp_end_timestamp(
                        file_identifier.clone(),
                        item.start_timestamp as i32,
                        item.end_timestamp as i32,
                    ),
                    (
                        file_identifier,
                        item.start_timestamp as i32,
                        item.end_timestamp as i32,
                        item.text.clone(),
                        vec![],
                    ),
                    vec![],
                );

                match x.exec().await {
                    std::result::Result::Ok(res) => {
                        let _ = save_text_embedding(
                            &item.text,
                            res.id as u64,
                            clip_model,
                            embedding_index,
                        )
                        .await;
                        debug!("transcript embedding saved");
                    }
                    Err(e) => {
                        error!("failed to save transcript embedding: {:?}", e);
                    }
                }
            });
        }

        while let Some(_) = join_set.join_next().await {}

        Ok(())
    }

    /// Get frame content embedding
    pub async fn get_frame_content_embedding(&self) -> anyhow::Result<()> {
        let clip_model = self.get_clip_instance().await?;
        let clip_model = Arc::new(RwLock::new(clip_model));

        let embedding_index = Arc::new(self.indexes.frame_index.clone());

        let frame_paths = std::fs::read_dir(&self.frames_dir)?
            .map(|res| res.map(|e| e.path()))
            .collect::<Result<Vec<_>, std::io::Error>>()?;

        let mut join_set = tokio::task::JoinSet::new();

        for path in frame_paths {
            if path.extension() == Some(std::ffi::OsStr::new("png")) {
                debug!("handle file: {:?}", path);

                let clip_model = clip_model.clone();
                let embedding_index = embedding_index.clone();
                let client = self.client.clone();

                let file_name = path
                    .file_name()
                    .ok_or(anyhow!("invalid path"))?
                    .to_str()
                    .ok_or(anyhow!("invalid path"))?;

                let frame_timestamp: i64 = file_name
                    .split(".")
                    .next()
                    .unwrap_or("0")
                    .parse()
                    .unwrap_or(0);

                let file_identifier = self.file_identifier.clone();
                let client = client.clone();

                join_set.spawn(async move {
                    // write data using prisma
                    let client = client.write().await;
                    let x = client.video_frame().upsert(
                        video_frame::file_identifier_timestamp(
                            file_identifier.clone(),
                            frame_timestamp as i32,
                        ),
                        (file_identifier.clone(), frame_timestamp as i32, vec![]),
                        vec![],
                    );

                    match x.exec().await {
                        std::result::Result::Ok(res) => {
                            let _ = get_single_frame_content_embedding(
                                res.id as u64,
                                path,
                                clip_model,
                                embedding_index,
                            )
                            .await;
                            debug!("frame content embedding saved");
                        }
                        Err(e) => {
                            error!("failed to save frame content embedding: {:?}", e);
                        }
                    }
                });
            }
        }

        // wait for all tasks
        while let Some(_) = join_set.join_next().await {}

        Ok(())
    }

    /// Get frames' captions of video
    /// this requires extracting frames in advance
    ///
    /// All caption will be saved in .caption file in the same place with frame file
    pub async fn get_frames_caption(&self) -> anyhow::Result<()> {
        let frame_paths = std::fs::read_dir(&self.frames_dir)?
            .map(|res| res.map(|e| e.path()))
            .collect::<Result<Vec<_>, std::io::Error>>()?;

        let blip_model = ai::blip::BLIP::new(&self.resources_dir).await?;
        let blip_model = Arc::new(RwLock::new(blip_model));

        let mut join_set = tokio::task::JoinSet::new();

        for path in frame_paths {
            if path.extension() == Some(std::ffi::OsStr::new("png")) {
                debug!("get_frames_caption: {:?}", path);
                let blip_model = Arc::clone(&blip_model);

                join_set.spawn(async move {
                    let _ = get_single_frame_caption(blip_model, path).await;
                });
            }
        }

        while let Some(_) = join_set.join_next().await {}

        Ok(())
    }

    /// Get frame caption embedding
    /// this requires extracting frames and get captions in advance
    pub async fn get_frame_caption_embedding(&self) -> anyhow::Result<()> {
        let clip_model = self.get_clip_instance().await?;
        let clip_model = Arc::new(RwLock::new(clip_model));

        let embedding_index = Arc::new(self.indexes.frame_caption_index.clone());

        let frame_paths = std::fs::read_dir(&self.frames_dir)?
            .map(|res| res.map(|e| e.path()))
            .collect::<Result<Vec<_>, std::io::Error>>()?;

        let mut join_set = tokio::task::JoinSet::new();

        for path in frame_paths {
            if path.extension() == Some(std::ffi::OsStr::new("caption")) {
                let clip_model = Arc::clone(&clip_model);
                let embedding_index = Arc::clone(&embedding_index);
                let file_identifier = self.file_identifier.clone();
                let client = self.client.clone();

                // FIXME 这里限制一下最大任务数量，因为出现过 axum 被 block 的情况
                if join_set.len() >= 3 {
                    while let Some(_) = join_set.join_next().await {}
                }

                join_set.spawn(async move {
                    let _ = get_single_frame_caption_embedding(
                        file_identifier,
                        client,
                        path,
                        clip_model,
                        embedding_index,
                    )
                    .await;
                });
            }
        }

        while let Some(_) = join_set.join_next().await {}

        Ok(())
    }

    async fn get_clip_instance(&self) -> anyhow::Result<ai::clip::CLIP> {
        ai::clip::CLIP::new(
            ai::clip::model::CLIPModel::ViTB32,
            &self.resources_dir,
        )
        .await
    }
}

async fn get_single_frame_caption(
    blip_model: Arc<RwLock<ai::blip::BLIP>>,
    path: impl AsRef<std::path::Path>,
) -> anyhow::Result<()> {
    let caption = blip_model.write().await.get_caption(path.as_ref()).await?;
    debug!("caption: {:?}", caption);

    // write into file
    let caption_path = path
        .as_ref()
        .to_str()
        .ok_or(anyhow!("invalid path"))?
        .replace(".png", ".caption");
    let mut file = tokio::fs::File::create(caption_path).await?;
    file.write_all(caption.as_bytes()).await?;

    Ok(())
}

async fn save_text_embedding(
    text: &str,
    id: u64,
    clip: Arc<RwLock<ai::clip::CLIP>>,
    embedding_index: Arc<EmbeddingIndex>,
) -> anyhow::Result<()> {
    let embedding = clip.read().await.get_text_embedding(text).await?;
    let embedding: Vec<f32> = embedding.iter().map(|&x| x).collect();

    embedding_index.add(id, embedding).await?;

    Ok(())
}

async fn get_single_frame_content_embedding(
    id: u64,
    path: impl AsRef<std::path::Path>,
    clip_model: Arc<RwLock<ai::clip::CLIP>>,
    embedding_index: Arc<EmbeddingIndex>,
) -> anyhow::Result<()> {
    let embedding = clip_model
        .read()
        .await
        .get_image_embedding_from_file(path.as_ref())
        .await?;
    let embedding: Vec<f32> = embedding.iter().map(|&x| x).collect();

    embedding_index.add(id, embedding).await?;

    Ok(())
}

async fn get_single_frame_caption_embedding(
    file_identifier: String,
    client: Arc<RwLock<PrismaClient>>,
    path: impl AsRef<std::path::Path>,
    clip_model: Arc<RwLock<ai::clip::CLIP>>,
    embedding_index: Arc<EmbeddingIndex>,
) -> anyhow::Result<()> {
    let caption = tokio::fs::read_to_string(path.as_ref()).await?;
    let file_name = path
        .as_ref()
        .file_name()
        .ok_or(anyhow!("invalid path"))?
        .to_str()
        .ok_or(anyhow!("invalid path"))?;

    let frame_timestamp: i64 = file_name
        .split(".")
        .next()
        .unwrap_or("0")
        .parse()
        .unwrap_or(0);

    let client = client.write().await;

    let video_frame = client
        .video_frame()
        .upsert(
            video_frame::UniqueWhereParam::FileIdentifierTimestampEquals(
                file_identifier.clone(),
                frame_timestamp as i32,
            ),
            (file_identifier.clone(), frame_timestamp as i32, vec![]),
            vec![],
        )
        .exec()
        .await?;

    let x = client.video_frame_caption().upsert(
        video_frame_caption::UniqueWhereParam::VideoFrameIdEquals(video_frame.id),
        (
            caption.clone(),
            video_frame::UniqueWhereParam::IdEquals(video_frame.id),
            vec![],
        ),
        vec![],
    );

    match x.exec().await {
        std::result::Result::Ok(res) => {
            save_text_embedding(&caption, res.id as u64, clip_model, embedding_index).await?;
        }
        Err(e) => {
            error!("failed to save frame caption embedding: {:?}", e);
        }
    }

    Ok(())
}

#[test_log::test(tokio::test)]
async fn test_handle_video() {
    let video_path = "/Users/zhuo/Desktop/file_v2_f566a493-ad1b-4324-b16f-0a4c6a65666g 2.MP4";
    // let video_path = "/Users/zhuo/Desktop/屏幕录制2022-11-30 11.43.29.mov";
    let local_data_dir = "/Users/zhuo/Library/Application Support/cc.musedam.local";
    let resources_dir = "/Users/zhuo/dev/bmrlab/tauri-dam-test-playground/target/debug/resources";

    let video_handler = VideoHandler::new(
        video_path,
        local_data_dir,
        resources_dir,
        "/Users/zhuo/Library/Application Support/cc.musedam.local/dev.db",
    )
    .await;

    if video_handler.is_err() {
        tracing::error!("failed to create video handler: {:?}", video_handler);
    }
    let video_handler = video_handler.unwrap();

    tracing::info!("file handler initialized");

    video_handler
        .get_frames()
        .await
        .expect("failed to get frames");

    tracing::info!("got frames");

    video_handler
        .get_frame_content_embedding()
        .await
        .expect("failed to get frame content embedding");

    tracing::debug!("got frame content embedding");

    video_handler
        .indexes
        .frame_index
        .flush()
        .await
        .expect("failed to flush index");

    video_handler
        .get_audio()
        .await
        .expect("failed to get audio");

    tracing::info!("got audio");

    video_handler
        .get_transcript()
        .await
        .expect("failed to get transcript");

    tracing::info!("got transcript");

    video_handler
        .get_transcript_embedding()
        .await
        .expect("failed to get transcript embedding");

    video_handler
        .indexes
        .transcript_index
        .flush()
        .await
        .expect("failed to flush index");

    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
}
