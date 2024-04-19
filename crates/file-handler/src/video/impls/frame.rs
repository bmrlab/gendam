use std::path::Path;

use prisma_lib::video_frame;
use qdrant_client::qdrant::{point_id::PointIdOptions, PointId, PointStruct};
use serde_json::json;
use tracing::{debug, error};

use crate::{
    search::payload::SearchPayload,
    video::{decoder, VideoHandler, FRAME_DIR, FRAME_FILE_EXTENSION},
};

use super::get_frame_timestamp_from_path;

impl VideoHandler {
    /// Extract key frames from video and save results
    /// - Save into disk (a folder named by `library` and `video_file_hash`)
    /// - Save into prisma `VideoFrame` model
    pub(crate) async fn save_frames(&self) -> anyhow::Result<()> {
        let video_path = &self.video_path;
        let frames_dir = self.artifacts_dir.join(FRAME_DIR);

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

        let client = self.library.prisma_client();
        let file_identifier = self.file_identifier();
        let frames_dir = self.artifacts_dir.join(FRAME_DIR);

        let frame_paths = std::fs::read_dir(frames_dir)?
            .filter_map(|res| res.map(|e| e.path()).ok())
            .filter(|v| v.extension() == Some(std::ffi::OsStr::new(FRAME_FILE_EXTENSION)))
            .collect::<Vec<_>>();

        for path in frame_paths {
            let client = client.clone();
            let frame_timestamp = get_frame_timestamp_from_path(&path)?;

            // write data using prisma
            let x = {
                client
                    .video_frame()
                    .upsert(
                        video_frame::file_identifier_timestamp(
                            file_identifier.to_string(),
                            frame_timestamp as i32,
                        ),
                        (file_identifier.to_string(), frame_timestamp as i32, vec![]),
                        vec![],
                    )
                    .exec()
                    .await
            };

            if let Err(e) = x {
                error!("failed to save frame content embedding: {:?}", e);
            }
        }

        Ok(())
    }

    /// Save frame content embedding into qdrant
    pub(crate) async fn save_frame_content_embedding(&self) -> anyhow::Result<()> {
        let frames_dir = self.artifacts_dir.join(FRAME_DIR);
        let client = self.library.prisma_client();
        let file_identifier = self.file_identifier();

        // 这里还是从本地读取所有图片
        // 因为可能一个视频包含的帧数可能非常多，从 sqlite 读取反而麻烦了
        let frame_paths = std::fs::read_dir(frames_dir)?
            .filter_map(|res| res.map(|e| e.path()).ok())
            .filter(|v| v.extension() == Some(std::ffi::OsStr::new(FRAME_FILE_EXTENSION)))
            .collect::<Vec<_>>();

        for path in frame_paths {
            let client = client.clone();

            let frame_timestamp = get_frame_timestamp_from_path(&path)?;

            // get data using prisma
            let x = {
                client
                    .video_frame()
                    .find_unique(video_frame::file_identifier_timestamp(
                        file_identifier.to_string(),
                        frame_timestamp as i32,
                    ))
                    .exec()
                    .await
                // drop the rwlock
            };

            match x {
                std::result::Result::Ok(Some(res)) => {
                    let payload = SearchPayload::Frame {
                        id: res.id as u64,
                        file_identifier: file_identifier.to_string(),
                        timestamp: frame_timestamp,
                    };

                    let _ = self
                        .get_single_frame_content_embedding(payload, &path)
                        .await;
                    debug!("frame content embedding saved");
                }
                std::result::Result::Ok(None) => {
                    error!("failed to find frame");
                }
                Err(e) => {
                    error!("failed to save frame content embedding: {:?}", e);
                }
            }
        }

        Ok(())
    }

    async fn get_single_frame_content_embedding(
        &self,
        payload: SearchPayload,
        path: impl AsRef<Path>,
    ) -> anyhow::Result<()> {
        let qdrant = self.library.qdrant_client();
        let multi_modal_embedding = self.multi_modal_embedding()?;
        let collection_name = self.vision_collection_name()?;

        // if point exists, skip
        match qdrant
            .get_points(
                collection_name,
                None,
                &[PointId {
                    point_id_options: Some(PointIdOptions::Uuid(payload.get_uuid().to_string())),
                }],
                Some(false),
                Some(false),
                None,
            )
            .await
        {
            std::result::Result::Ok(res) if res.result.len() > 0 => {
                debug!("frame content embedding already exists, skip it");
                return Ok(());
            }
            _ => {}
        }

        let embedding = multi_modal_embedding
            .get_images_embedding_tx()
            .process_single(path.as_ref().to_path_buf())
            .await?;

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

    pub(crate) async fn delete_frames(&self) -> anyhow::Result<()> {
        let client = self.library.prisma_client();
        let file_identifier = self.file_identifier();

        client
            .video_frame()
            .delete_many(vec![video_frame::file_identifier::equals(
                file_identifier.to_string(),
            )])
            .exec()
            .await?;

        // delete frame files on disk
        let frames_dir = self.artifacts_dir.join(FRAME_DIR);
        if frames_dir.exists() {
            std::fs::remove_dir_all(&frames_dir)?;
        }

        Ok(())
    }

    /// Delete frame content embedding in qdrant
    pub(crate) async fn delete_frame_content_embedding(&self) -> anyhow::Result<()> {
        self.delete_embedding(crate::SearchRecordType::Frame).await
    }
}
