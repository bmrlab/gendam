use crate::{
    search::payload::SearchPayload,
    video::{
        impls::get_frame_timestamp_from_path, VideoHandler, VideoTaskType, CAPTION_FILE_EXTENSION,
        EMBEDDING_FILE_EXTENSION,
    },
};
use qdrant_client::qdrant::PointStruct;
use serde_json::{json, Value};
use std::path::PathBuf;
use storage::prelude::*;
use tracing::{debug, error};

impl VideoHandler {
    fn get_frames_caption_dir(&self) -> anyhow::Result<PathBuf> {
        let output_path = self
            .get_output_info_in_settings(&VideoTaskType::FrameCaption)?
            .dir;
        let dir = self.artifacts_dir.join(output_path);
        Ok(dir)
    }

    pub fn get_frame_caption_path(&self, timestamp: i64) -> anyhow::Result<PathBuf> {
        Ok(self
            .get_frames_caption_dir()?
            .join(format!("{}.{}", timestamp, CAPTION_FILE_EXTENSION)))
    }

    pub fn get_frame_caption(&self, timestamp: i64) -> anyhow::Result<String> {
        let path = self.get_frame_caption_path(timestamp)?;
        let content_str = self.read_to_string(path)?;
        let json_string: Value = serde_json::from_str(&content_str)?;
        let caption = json_string["caption"]
            .as_str()
            .ok_or(anyhow::anyhow!("no caption found in frame caption file"))?;

        Ok(caption.to_string())
    }

    pub fn get_frame_caption_embedding_path(&self, timestamp: i64) -> anyhow::Result<PathBuf> {
        let output_path = self
            .get_output_info_in_settings(&VideoTaskType::FrameCaptionEmbedding)?
            .dir;

        Ok(self
            .artifacts_dir
            .join(output_path)
            .join(format!("{}.{}", timestamp, EMBEDDING_FILE_EXTENSION)))
    }

    pub fn get_frame_caption_embedding(&self, timestamp: i64) -> anyhow::Result<Vec<f32>> {
        let embedding_path = self.get_frame_caption_embedding_path(timestamp)?;
        self.get_embedding_from_file(embedding_path)
    }

    /// Save frames' captions of video
    /// **this requires extracting frames in advance**
    ///
    /// The captions will be saved:
    /// - To disk: as `.caption` file in the same place with frame file
    /// - To prisma `VideoFrameCaption` model
    pub(crate) async fn save_frames_caption(&self) -> anyhow::Result<()> {
        let (image_caption, _) = self.image_caption()?;
        let frame_paths = self.list_frame_paths().await?;
        for path in frame_paths {
            debug!("get_frames_caption: {:?}", path);

            let timestamp = get_frame_timestamp_from_path(&path)?;

            if self.get_frame_caption(timestamp).is_ok() {
                continue;
            }

            let caption_path = self.get_frame_caption_path(timestamp)?;
            let caption = image_caption
                .get_images_caption_tx()
                .process_single(path)
                .await?;

            // write into file

            self.write(
                caption_path,
                json!({
                    "caption": caption
                })
                .to_string()
                .into(),
            )
            .await?;
        }

        Ok(())
    }

    pub(crate) async fn save_frame_caption_embedding(&self) -> anyhow::Result<()> {
        let frame_paths = self.list_frame_paths().await?;

        for path in frame_paths {
            debug!("save_frame_caption_embedding: {:?}", path,);

            let timestamp = get_frame_timestamp_from_path(path)?;

            if self.get_frame_caption_embedding(timestamp).is_err() {
                let caption = self.get_frame_caption(timestamp)?;
                let path = self.get_frame_caption_embedding_path(timestamp)?;
                if let Err(e) = self.save_text_embedding(&caption, path).await {
                    error!("failed to save frame caption embedding: {:?}", e);
                }
            }

            self.save_db_single_frame_caption_embedding(timestamp)
                .await?;
        }

        Ok(())
    }

    pub(crate) async fn save_db_single_frame_caption_embedding(
        &self,
        timestamp: i64,
    ) -> anyhow::Result<()> {
        let qdrant = self.qdrant_client()?;
        let collection_name = self.language_collection_name()?;

        let embedding = self.get_frame_caption_embedding(timestamp)?;

        let image_caption_model_name = self.image_caption()?.1;

        let payload = SearchPayload::FrameCaption {
            file_identifier: self.file_identifier().to_string(),
            timestamp,
            method: image_caption_model_name.into(),
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
