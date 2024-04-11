use crate::{search::payload::SearchPayload, video::FRAME_FILE_EXTENSION};
use ai::{
    clip::{CLIPInput, CLIP},
    BatchHandler,
};
use anyhow::Ok;
use prisma_lib::{video_frame, PrismaClient};
use qdrant_client::{client::QdrantClient, qdrant::PointStruct};
use serde_json::json;
use std::sync::Arc;
use tracing::{debug, error};

use super::get_frame_timestamp_from_path;

pub async fn save_frames(
    file_identifier: String,
    client: Arc<PrismaClient>,
    frames_dir: impl AsRef<std::path::Path>,
) -> anyhow::Result<()> {
    let frame_paths = std::fs::read_dir(frames_dir.as_ref())?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, std::io::Error>>()?;

    for path in frame_paths {
        if path.extension() == Some(std::ffi::OsStr::new(FRAME_FILE_EXTENSION)) {
            let client = client.clone();
            let frame_timestamp = get_frame_timestamp_from_path(&path)?;
            let file_identifier = file_identifier.clone();

            // write data using prisma
            let x = {
                client
                    .video_frame()
                    .upsert(
                        video_frame::file_identifier_timestamp(
                            file_identifier.clone(),
                            frame_timestamp as i32,
                        ),
                        (file_identifier.clone(), frame_timestamp as i32, vec![]),
                        vec![],
                    )
                    .exec()
                    .await
            };

            if let Err(e) = x {
                error!("failed to save frame content embedding: {:?}", e);
            }
        }
    }

    Ok(())
}

pub async fn save_frame_content_embedding(
    file_identifier: String,
    client: Arc<PrismaClient>,
    frames_dir: impl AsRef<std::path::Path>,
    clip_model: BatchHandler<CLIP>,
    qdrant: Arc<QdrantClient>,
) -> anyhow::Result<()> {
    // 这里还是从本地读取所有图片
    // 因为可能一个视频包含的帧数可能非常多，从 sqlite 读取反而麻烦了
    let frame_paths = std::fs::read_dir(frames_dir.as_ref())?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, std::io::Error>>()?;

    for path in frame_paths {
        if path.extension() == Some(std::ffi::OsStr::new(FRAME_FILE_EXTENSION)) {
            let client = client.clone();
            let clip_model = clip_model.clone();
            let qdrant = qdrant.clone();

            let frame_timestamp = get_frame_timestamp_from_path(&path)?;
            let file_identifier = file_identifier.clone();

            // get data using prisma
            let x = {
                client
                    .video_frame()
                    .find_unique(video_frame::file_identifier_timestamp(
                        file_identifier.clone(),
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
                        file_identifier: file_identifier.clone(),
                        timestamp: frame_timestamp,
                    };

                    let _ = get_single_frame_content_embedding(payload, &path, clip_model, qdrant)
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
    }

    Ok(())
}

async fn get_single_frame_content_embedding(
    payload: SearchPayload,
    path: impl AsRef<std::path::Path>,
    clip_model: BatchHandler<CLIP>,
    qdrant: Arc<QdrantClient>,
) -> anyhow::Result<()> {
    let embedding = clip_model
        .process_single(CLIPInput::ImageFilePath(path.as_ref().to_path_buf()))
        .await?;
    let embedding: Vec<f32> = embedding.iter().map(|&x| x).collect();

    let point = PointStruct::new(
        payload.get_uuid().to_string(),
        embedding.clone(),
        json!(payload)
            .try_into()
            .map_err(|_| anyhow::anyhow!("invalid payload"))?,
    );
    qdrant
        .upsert_points(
            vector_db::DEFAULT_VISION_COLLECTION_NAME,
            None,
            vec![point],
            None,
        )
        .await?;

    Ok(())
}
