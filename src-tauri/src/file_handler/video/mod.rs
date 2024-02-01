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
use tokio::io::AsyncWriteExt;
use tokio::task::JoinSet;
use tracing::debug;

mod decoder;

#[derive(Clone, Debug)]
pub struct VideoHandler {
    video_path: std::path::PathBuf,
    artifacts_dir: std::path::PathBuf,
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
    pub fn new(
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

        Ok(Self {
            video_path: video_path.as_ref().to_owned(),
            artifacts_dir: artifacts_dir.clone(),
            resources_dir: resources_dir.as_ref().to_owned(),
            file_identifier: file_sha256.clone().into(),
            audio_path: artifacts_dir.join("audio.wav"),
            frames_dir,
            transcript_path: artifacts_dir.join("transcript.txt"),
        })
    }

    /// Trigger the video handler, to save artifacts (frame, audio, etc.)
    pub fn decode_video(&mut self) -> anyhow::Result<()> {
        let mut video_decoder = decoder::video::VideoDecoder::new(&self.video_path)?;
        video_decoder.save_video_artifacts(&self.frames_dir, &self.audio_path)?;

        Ok(())
    }

    pub async fn get_transcript(&self) -> anyhow::Result<()> {
        let mut whisper =
            audio::AudioWhisper::new(self.resources_dir.join("whisper-ggml-base.bin"))?;
        let result = whisper.transcribe(&self.audio_path, &self.transcript_path)?;

        // write results into json file
        let mut file = tokio::fs::File::create(&self.transcript_path).await?;
        let json = serde_json::to_string(&result)?;
        file.write_all(json.as_bytes()).await?;

        Ok(())
    }

    pub async fn get_transcript_embedding(&self) -> anyhow::Result<()> {
        let file = File::open(&self.transcript_path)?;
        let reader = BufReader::new(file);

        // Read the JSON contents of the file as an instance of `User`.
        let whisper_results: Vec<WhisperItem> = serde_json::from_reader(reader)?;

        let mut clip_model = self.get_clip_instance()?;
        let qdrant_client = self.get_qdrant_client()?;

        for item in whisper_results {
            self.get_single_transcript_embedding(item, &mut clip_model, &qdrant_client)
                .await?;
        }

        Ok(())
    }

    async fn get_single_transcript_embedding(
        &self,
        item: WhisperItem,
        clip_model: &mut embedding::clip::CLIP,
        qdrant_client: &QdrantClient,
    ) -> anyhow::Result<()> {
        let payload = SearchPayload::Transcript(search_payload::TranscriptPayload {
            file_identifier: self.file_identifier.clone(),
            start_timestamp: item.start_timestamp,
            end_timestamp: item.end_timestamp,
            transcript: item.text.clone(),
        });

        self.save_text_embedding(&item.text, payload, clip_model, qdrant_client)
            .await?;

        Ok(())
    }

    /// Get frame content embedding
    pub async fn get_frame_content_embedding(&self) -> anyhow::Result<()> {
        let mut clip_model = self.get_clip_instance()?;
        let qdrant_client = self.get_qdrant_client()?;

        let frame_paths = std::fs::read_dir(&self.frames_dir)?
            .map(|res| res.map(|e| e.path()))
            .collect::<Result<Vec<_>, std::io::Error>>()?;

        for path in frame_paths {
            if path.extension() == Some(std::ffi::OsStr::new("png")) {
                self.get_single_frame_content_embedding(path, &mut clip_model, &qdrant_client)
                    .await?;
            }
        }

        Ok(())
    }

    async fn get_single_frame_content_embedding(
        &self,
        path: impl AsRef<std::path::Path>,
        clip_model: &mut embedding::clip::CLIP,
        qdrant_client: &QdrantClient,
    ) -> anyhow::Result<()> {
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

        let payload = SearchPayload::Frame(search_payload::FramePayload {
            file_identifier: self.file_identifier.clone(),
            frame_filename: file_name.to_string(),
            timestamp: frame_timestamp,
        });

        let embedding = clip_model
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
            .upsert_points(super::QDRANT_COLLECTION_NAME, None, vec![point], None)
            .await?;

        todo!()
    }

    pub async fn get_frames_caption(&self) -> anyhow::Result<()> {
        let frame_paths = std::fs::read_dir(&self.frames_dir)?
            .map(|res| res.map(|e| e.path()))
            .collect::<Result<Vec<_>, std::io::Error>>()?;

        let mut tokio_join_set = JoinSet::new();
        let blip_model_dir = self.resources_dir.join("blip");

        for path in frame_paths {
            if path.extension() == Some(std::ffi::OsStr::new("png")) {
                debug!("get_frames_caption: {:?}", path);
                let blip_dir = blip_model_dir.clone();
                tokio_join_set.spawn(async move {
                    let _ = get_single_frame_caption(blip_dir, path.clone()).await;
                });
            }
        }
        while let Some(_) = tokio_join_set.join_next().await {}

        Ok(())
    }

    pub async fn get_frame_caption_embedding(&self) -> anyhow::Result<()> {
        let mut clip_model = self.get_clip_instance()?;
        let qdrant_client = self.get_qdrant_client()?;

        let frame_paths = std::fs::read_dir(&self.frames_dir)?
            .map(|res| res.map(|e| e.path()))
            .collect::<Result<Vec<_>, std::io::Error>>()?;

        for path in frame_paths {
            if path.ends_with(".caption") {
                self.get_single_frame_caption_embedding(path, &mut clip_model, &qdrant_client)
                    .await?;
            }
        }

        Ok(())
    }

    async fn get_single_frame_caption_embedding(
        &self,
        path: impl AsRef<std::path::Path>,
        clip_model: &mut embedding::clip::CLIP,
        qdrant_client: &QdrantClient,
    ) -> anyhow::Result<()> {
        let caption = fs::read_to_string(path.as_ref())?;
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
            file_identifier: self.file_identifier.clone(),
            frame_filename: file_name.to_string(),
            caption: caption.clone(),
            timestamp: frame_timestamp,
        });

        self.save_text_embedding(&caption, payload, clip_model, qdrant_client);

        todo!()
    }

    async fn save_text_embedding(
        &self,
        text: &str,
        payload: SearchPayload,
        clip: &embedding::clip::CLIP,
        qdrant_client: &QdrantClient,
    ) -> anyhow::Result<()> {
        let embedding = clip.get_text_embedding(text).await?;
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
            .upsert_points(super::QDRANT_COLLECTION_NAME, None, vec![point], None)
            .await?;

        Ok(())
    }

    fn get_clip_instance(&self) -> anyhow::Result<embedding::clip::CLIP> {
        embedding::clip::CLIP::new(
            self.resources_dir
                .join("visual.onnx")
                .to_str()
                .ok_or(anyhow::anyhow!("invalid path"))?,
            self.resources_dir
                .join("textual.onnx")
                .to_str()
                .ok_or(anyhow::anyhow!("invalid path"))?,
            self.resources_dir
                .join("tokenizer.json")
                .to_str()
                .ok_or(anyhow::anyhow!("invalid path"))?,
        )
    }

    fn get_qdrant_client(&self) -> anyhow::Result<QdrantClient> {
        QdrantClientConfig::from_url("http://0.0.0.0:6334").build()
    }
}

#[test_log::test]
fn test_handle_video() {
    todo!()
}

async fn get_single_frame_caption(
    blip_model_dir: impl AsRef<std::path::Path>,
    path: impl AsRef<std::path::Path>,
) -> anyhow::Result<()> {
    let mut blip_model = embedding::blip::BLIP::new(blip_model_dir)?;
    let caption = blip_model.get_caption(path.as_ref()).await?;
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
