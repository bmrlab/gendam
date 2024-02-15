use super::audio::WhisperItem;
use super::embedding;
use super::search_payload;
use super::{audio, search_payload::SearchPayload};
use anyhow::{anyhow, Ok};
use ndarray::Axis;
use qdrant_client::client::QdrantClient;
use qdrant_client::client::QdrantClientConfig;
use qdrant_client::qdrant::PointStruct;
use serde_json::json;
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::sync::RwLock;
use futures::future::join_all;
use tracing::debug;

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
///
/// let video_handler = VideoHandler::new(
///     video_path,
///     local_data_dir,
///     resources_dir,
/// ).await.unwrap();
///
/// // get frames and save their embedding into qdrant
/// video_handler.get_frames().await;
/// video_handler.get_frame_content_embedding().await;
///
/// // get audio, then extract text, and finally save text embedding in qdrant
/// video_handler.get_audio().await;
/// video_handler.get_transcript().await;
/// video_handler.get_transcript_embedding().await;
/// ```
#[derive(Clone, Debug)]
pub struct VideoHandler {
    video_path: std::path::PathBuf,
    resources_dir: std::path::PathBuf,
    file_identifier: String,
    frames_dir: std::path::PathBuf,
    audio_path: std::path::PathBuf,
    transcript_path: std::path::PathBuf,
}

impl VideoHandler {
    /// Create a new VideoHandler
    ///
    /// # Arguments
    ///
    /// * `video_path` - The path to the video file
    /// * `local_data_dir` - The path to the local data directory, where artifacts (frame, transcript, etc.) will be saved into
    /// * `resources_dir` - The path to the resources directory (src-tauri/resources), where contains model files
    pub async fn new(
        video_path: impl AsRef<std::path::Path>,
        local_data_dir: impl AsRef<std::path::Path>,
        resources_dir: impl AsRef<std::path::Path>,
    ) -> anyhow::Result<Self> {
        let bytes = std::fs::read(&video_path)?;
        let file_sha256 = sha256::digest(&bytes);
        let artifacts_dir = local_data_dir.as_ref().join(&file_sha256);
        let frames_dir = artifacts_dir.join("frames");

        fs::create_dir_all(&artifacts_dir)?;
        fs::create_dir_all(&frames_dir)?;

        // make sure clip files are downloaded
        let resources_dir_clone = resources_dir.as_ref().to_owned();
        let clip_handle = tokio::spawn(embedding::clip::CLIP::new(
            embedding::clip::model::CLIPModel::ViTB32,
            resources_dir_clone,
        ));

        let resources_dir_clone = resources_dir.as_ref().to_owned();
        let whisper_handle = tokio::spawn(audio::AudioWhisper::new(resources_dir_clone));

        let clip_result = clip_handle.await;
        let whisper_result = whisper_handle.await;

        if let Err(clip_err) = clip_result.unwrap() {
            return Err(anyhow!("Failed to download clip model: {clip_err}"));
        }
        if let Err(whisper_err) = whisper_result.unwrap() {
            return Err(anyhow!("Failed to download whisper model: {whisper_err}"));
        }

        debug!("clip and whisper models downloaded");

        Ok(Self {
            video_path: video_path.as_ref().to_owned(),
            resources_dir: resources_dir.as_ref().to_owned(),
            file_identifier: file_sha256.clone(),
            audio_path: artifacts_dir.join("audio.wav"),
            frames_dir,
            transcript_path: artifacts_dir.join("transcript.txt"),
        })
    }

    pub fn file_identifier(&self) -> &str {
        &self.file_identifier
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
        let mut whisper = audio::AudioWhisper::new(&self.resources_dir).await?;
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

        let qdrant_client = self.get_qdrant_client()?;
        let qdrant_client = Arc::new(RwLock::new(qdrant_client));

        for item in whisper_results {
            // if item is some like [MUSIC], just skip it
            // TODO need to make sure all filter rules
            if item.text.starts_with("[") && item.text.ends_with("]") {
                continue;
            }

            let clip_model = Arc::clone(&clip_model);
            let qdrant_client = Arc::clone(&qdrant_client);
            let file_identifier = self.file_identifier.clone();

            tokio::spawn(async move {
                let payload = SearchPayload::Transcript(search_payload::TranscriptPayload {
                    file_identifier,
                    start_timestamp: item.start_timestamp,
                    end_timestamp: item.end_timestamp,
                    transcript: item.text.clone(),
                });
                let _ = save_text_embedding(&item.text, payload, clip_model, qdrant_client).await;
            });
        }

        Ok(())
    }

    /// Get frame content embedding
    pub async fn get_frame_content_embedding(&self) -> anyhow::Result<()> {
        let clip_model = self.get_clip_instance().await?;
        let clip_model = Arc::new(RwLock::new(clip_model));

        let qdrant_client = self.get_qdrant_client()?;
        let qdrant_client = Arc::new(RwLock::new(qdrant_client));

        let frame_paths = std::fs::read_dir(&self.frames_dir)?
            .map(|res| res.map(|e| e.path()))
            .collect::<Result<Vec<_>, std::io::Error>>()?;

        for path in frame_paths {
            if path.extension() == Some(std::ffi::OsStr::new("png")) {
                let clip_model = Arc::clone(&clip_model);
                let qdrant_client = Arc::clone(&qdrant_client);
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

                let payload = SearchPayload::Frame(search_payload::FramePayload {
                    file_identifier: self.file_identifier.clone(),
                    frame_filename: file_name.to_string(),
                    timestamp: frame_timestamp,
                });

                tokio::spawn(async move {
                    let _ = get_single_frame_content_embedding(
                        payload,
                        path,
                        clip_model,
                        qdrant_client,
                    )
                    .await;
                });
            }
        }

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

        let blip_model = embedding::blip::BLIP::new(&self.resources_dir).await?;
        let blip_model = Arc::new(RwLock::new(blip_model));

        // ToDo: 下面这么写有问题，因为一下子起来太多 async 任务似乎就 block axum 处理的请求了，有点奇怪
        // let tasks = frame_paths.iter()
        //     .filter(|path| path.extension() == Some(std::ffi::OsStr::new("png")))
        //     .map(|path| {
        //         debug!("get_frames_caption: {:?}", path);
        //         let blip_model = Arc::clone(&blip_model);
        //         get_single_frame_caption(blip_model, path)
        //     })
        //     .collect::<Vec<_>>();
        // join_all(tasks).await;

        let mut tasks: Vec<_> = vec![];

        for path in frame_paths {
            if path.extension() == Some(std::ffi::OsStr::new("png")) {
                debug!("get_frames_caption: {:?}", path);
                let blip_model = Arc::clone(&blip_model);
                let task = get_single_frame_caption(blip_model, path);
                tasks.push(task);
                if tasks.len() >= 3 {
                    join_all(tasks).await;
                    tasks = vec![]; // `tasks` is moved to join_all, so drop it and assign a new vec to it
                }
                // tokio::spawn(async move {
                //     let _ = get_single_frame_caption(blip_model, path).await;
                // });
            }
        }
        join_all(tasks).await;
        Ok(())
    }

    /// Get frame caption embedding
    /// this requires extracting frames and get captions in advance
    pub async fn get_frame_caption_embedding(&self) -> anyhow::Result<()> {
        let clip_model = self.get_clip_instance().await?;
        let clip_model = Arc::new(RwLock::new(clip_model));

        let qdrant_client = self.get_qdrant_client()?;
        let qdrant_client = Arc::new(RwLock::new(qdrant_client));

        let frame_paths = std::fs::read_dir(&self.frames_dir)?
            .map(|res| res.map(|e| e.path()))
            .collect::<Result<Vec<_>, std::io::Error>>()?;

        for path in frame_paths {
            if path.extension() == Some(std::ffi::OsStr::new("caption")) {
                let clip_model = Arc::clone(&clip_model);
                let qdrant_client = Arc::clone(&qdrant_client);
                let file_identifier = self.file_identifier.clone();

                tokio::spawn(async move {
                    let _ = get_single_frame_caption_embedding(
                        file_identifier,
                        path,
                        clip_model,
                        qdrant_client,
                    )
                    .await;
                });
            }
        }

        Ok(())
    }

    async fn get_clip_instance(&self) -> anyhow::Result<embedding::clip::CLIP> {
        embedding::clip::CLIP::new(
            embedding::clip::model::CLIPModel::ViTB32,
            &self.resources_dir,
        )
        .await
    }

    fn get_qdrant_client(&self) -> anyhow::Result<QdrantClient> {
        QdrantClientConfig::from_url("http://0.0.0.0:6334").build()
    }
}

async fn get_single_frame_caption(
    blip_model: Arc<RwLock<embedding::blip::BLIP>>,
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
    payload: SearchPayload,
    clip: Arc<RwLock<embedding::clip::CLIP>>,
    qdrant_client: Arc<RwLock<QdrantClient>>,
) -> anyhow::Result<()> {
    let embedding = clip.read().await.get_text_embedding(text).await?;
    let embedding: Vec<f32> = embedding
        .index_axis(Axis(0), 0)
        .iter()
        .map(|&x| x)
        .collect();

    let point = PointStruct::new(
        payload.uuid(),
        embedding,
        json!(payload)
            .try_into()
            .map_err(|_| anyhow!("invalid payload"))?,
    );
    qdrant_client
        .read()
        .await
        .upsert_points(super::QDRANT_COLLECTION_NAME, None, vec![point], None)
        .await?;

    Ok(())
}

async fn get_single_frame_content_embedding(
    payload: SearchPayload,
    path: impl AsRef<std::path::Path>,
    clip_model: Arc<RwLock<embedding::clip::CLIP>>,
    qdrant_client: Arc<RwLock<QdrantClient>>,
) -> anyhow::Result<()> {
    let embedding = clip_model
        .read()
        .await
        .get_image_embedding_from_file(path.as_ref())
        .await?;
    let embedding: Vec<f32> = embedding
        .index_axis(Axis(0), 0)
        .iter()
        .map(|&x| x)
        .collect();

    let point = PointStruct::new(
        payload.uuid(),
        embedding,
        json!(payload)
            .try_into()
            .map_err(|_| anyhow!("invalid payload"))?,
    );
    qdrant_client
        .read()
        .await
        .upsert_points(super::QDRANT_COLLECTION_NAME, None, vec![point], None)
        .await?;

    Ok(())
}

async fn get_single_frame_caption_embedding(
    file_identifier: String,
    path: impl AsRef<std::path::Path>,
    clip_model: Arc<RwLock<embedding::clip::CLIP>>,
    qdrant_client: Arc<RwLock<QdrantClient>>,
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

    let payload = SearchPayload::FrameCaption(search_payload::FrameCaptionPayload {
        file_identifier,
        frame_filename: file_name.to_string(),
        caption: caption.clone(),
        timestamp: frame_timestamp,
    });

    save_text_embedding(&caption, payload, clip_model, qdrant_client).await?;

    Ok(())
}

#[test_log::test]
fn test_handle_video() {
    todo!()
}
