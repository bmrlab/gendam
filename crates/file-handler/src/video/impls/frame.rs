use super::get_frame_timestamp_from_path;
use crate::{
    search::payload::SearchPayload,
    video::{decoder, VideoHandler, VideoTaskType, EMBEDDING_FILE_EXTENSION, FRAME_FILE_EXTENSION},
};
use qdrant_client::qdrant::PointStruct;
use serde_json::json;
use std::path::PathBuf;
use storage::prelude::*;

impl VideoHandler {
    pub(crate) fn get_frames_dir(&self) -> anyhow::Result<PathBuf> {
        let output_info = self.get_output_info_in_settings(&VideoTaskType::Frame)?;
        let dir = self.artifacts_dir.join(output_info.dir);
        Ok(dir)
    }

    pub fn get_frame_path(&self, timestamp: i64) -> anyhow::Result<PathBuf> {
        let path = self
            .get_frames_dir()?
            .join(format!("{}.{}", timestamp, FRAME_FILE_EXTENSION));

        Ok(path)
    }

    pub fn get_frame_embedding_path(&self, timestamp: i64) -> anyhow::Result<PathBuf> {
        let output_path = self
            .get_output_info_in_settings(&VideoTaskType::FrameContentEmbedding)?
            .dir;

        Ok(self
            .artifacts_dir
            .join(output_path)
            .join(format!("{}.{}", timestamp, EMBEDDING_FILE_EXTENSION)))
    }

    pub fn get_frame_embedding(&self, timestamp: i64) -> anyhow::Result<Vec<f32>> {
        let embedding_path = self.get_frame_embedding_path(timestamp)?;
        self.get_embedding_from_file(embedding_path)
    }

    pub async fn list_frame_paths(&self) -> anyhow::Result<Vec<PathBuf>> {
        let frames_dir = self.get_frames_dir()?;
        let frame_paths = self
            .read_dir(frames_dir)
            .await?
            .into_iter()
            .filter(|v| v.extension() == Some(std::ffi::OsStr::new(FRAME_FILE_EXTENSION)))
            .collect();
        Ok(frame_paths)
    }

    /// Extract key frames from video and save results
    /// - Save into disk (a folder named by `library` and `video_file_hash`)
    /// - Save into prisma `VideoFrame` model
    pub(crate) async fn save_frames(&self) -> anyhow::Result<()> {
        if self.check_artifacts(&VideoTaskType::Frame) {
            return Ok(());
        }

        let video_path = &self.video_path;
        let frames_dir = self.get_frames_dir()?;

        #[cfg(feature = "ffmpeg-binary")]
        {
            let video_decoder = decoder::VideoDecoder::new(video_path)?;
            video_decoder.save_video_frames(frames_dir.clone()).await?;
        }

        #[cfg(feature = "ffmpeg-dylib")]
        {
            let video_decoder = decoder::VideoDecoder::new(video_path);
            video_decoder.save_video_frames(frames_dir.clone()).await?;
        }

        Ok(())
    }

    /// Save frame content embedding into qdrant
    pub(crate) async fn save_frame_content_embedding(&self) -> anyhow::Result<()> {
        // 这里还是从本地读取所有图片
        // 因为一个视频包含的帧数可能非常多，从 sqlite 读取反而麻烦了
        let frame_paths = self.list_frame_paths().await?;
        let (multi_modal_embedding, _) = self.multi_modal_embedding()?;

        for path in frame_paths {
            let timestamp = get_frame_timestamp_from_path(&path)?;
            if self.get_frame_embedding(timestamp).is_err() {
                let embedding_path = self.get_frame_embedding_path(timestamp)?;
                let embedding = multi_modal_embedding
                    .get_images_embedding_tx()
                    .process_single(path)
                    .await?;

                self.write(embedding_path, serde_json::to_string(&embedding)?.into())
                    .await?;
            }

            self.save_db_single_frame_content_embedding(timestamp)
                .await?;
        }

        Ok(())
    }

    pub(crate) async fn save_db_single_frame_content_embedding(
        &self,
        timestamp: i64,
    ) -> anyhow::Result<()> {
        let qdrant = self.qdrant_client()?;
        let collection_name = self.vision_collection_name()?;

        let embedding = self.get_frame_embedding(timestamp)?;

        let payload = SearchPayload::Frame {
            file_identifier: self.file_identifier.to_string(),
            timestamp,
        };

        let point = PointStruct::new(
            payload.get_uuid().to_string(),
            embedding.clone(),
            json!(payload)
                .try_into()
                .map_err(|_| anyhow::anyhow!("invalid payload"))?,
        );
        qdrant
            .upsert_points(collection_name, None, vec![point], None)
            .await?;

        Ok(())
    }
}
