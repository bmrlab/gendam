use super::save_text_embedding;
use crate::{
    search_payload::SearchPayload,
    video::{utils::get_frame_timestamp_from_path, CAPTION_FILE_EXTENSION, FRAME_FILE_EXTENSION},
};
use ai::{blip::BLIP, clip::CLIP, BatchHandler};
use anyhow::Ok;
use prisma_lib::{video_frame, video_frame_caption, PrismaClient};
use qdrant_client::client::QdrantClient;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tracing::{debug, error};

pub async fn save_frames_caption<'a>(
    file_identifier: String,
    frames_dir: impl AsRef<std::path::Path>,
    blip_model: BatchHandler<BLIP>,
    client: Arc<PrismaClient>,
) -> anyhow::Result<()> {
    let frame_paths = std::fs::read_dir(frames_dir.as_ref())?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, std::io::Error>>()?;

    let mut join_set = tokio::task::JoinSet::new();

    for path in frame_paths {
        if path.extension() == Some(std::ffi::OsStr::new(FRAME_FILE_EXTENSION)) {
            debug!("get_frames_caption: {:?}", path);
            let blip_model = blip_model.clone();
            let frame_timestamp = get_frame_timestamp_from_path(&path)?;
            let client = client.clone();
            let file_identifier = file_identifier.clone();

            join_set.spawn(async move {
                match save_single_frame_caption(blip_model, path).await {
                    anyhow::Result::Ok(caption) => {
                        match client
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
                            .await
                        {
                            anyhow::Result::Ok(video_frame) => {
                                if let Err(e) = client
                                    .video_frame_caption()
                                    .upsert(
                                        video_frame_caption::UniqueWhereParam::VideoFrameIdEquals(
                                            video_frame.id,
                                        ),
                                        (
                                            caption.clone(),
                                            video_frame::UniqueWhereParam::IdEquals(video_frame.id),
                                            vec![],
                                        ),
                                        vec![],
                                    )
                                    .exec()
                                    .await
                                {
                                    error!("failed to upsert video frame caption: {}", e);
                                }
                            }
                            Err(e) => {
                                error!("failed to upsert video frame: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        error!("failed to get frame caption: {:?}", e);
                    }
                }
            });
        }
    }

    while let Some(_) = join_set.join_next().await {}

    Ok(())
}

async fn save_single_frame_caption(
    blip_handler: BatchHandler<BLIP>,
    path: impl AsRef<std::path::Path>,
) -> anyhow::Result<String> {
    let caption = blip_handler
        .process_single(path.as_ref().to_owned())
        .await?;

    debug!("caption: {:?}", caption);

    // write into file
    let caption_path = path
        .as_ref()
        .with_extension(CAPTION_FILE_EXTENSION)
        .to_string_lossy()
        .to_string();
    let mut file = tokio::fs::File::create(caption_path).await?;
    file.write_all(caption.as_bytes()).await?;

    Ok(caption)
}

pub async fn save_frame_caption_embedding(
    file_identifier: String,
    client: Arc<PrismaClient>,
    frames_dir: impl AsRef<std::path::Path>,
    clip_model: BatchHandler<CLIP>,
    qdrant: Arc<QdrantClient>,
) -> anyhow::Result<()> {
    let frame_paths = std::fs::read_dir(&frames_dir)?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, std::io::Error>>()?;

    let mut join_set = tokio::task::JoinSet::new();

    for path in frame_paths {
        if path.extension() == Some(std::ffi::OsStr::new("caption")) {
            let file_identifier = file_identifier.clone();
            let client = client.clone();
            let qdrant = qdrant.clone();

            let clip_model = clip_model.clone();

            join_set.spawn(async move {
                if let Err(e) = get_single_frame_caption_embedding(
                    file_identifier,
                    client,
                    path,
                    clip_model,
                    qdrant,
                )
                .await
                {
                    error!("failed to save frame caption embedding: {:?}", e);
                }
            });
        }
    }

    while let Some(_) = join_set.join_next().await {}

    Ok(())
}

async fn get_single_frame_caption_embedding(
    file_identifier: String,
    client: Arc<PrismaClient>,
    path: impl AsRef<std::path::Path>,
    clip_model: BatchHandler<CLIP>,
    qdrant: Arc<QdrantClient>,
) -> anyhow::Result<()> {
    let caption = tokio::fs::read_to_string(path.as_ref()).await?;
    let frame_timestamp = get_frame_timestamp_from_path(path)?;

    let x = {
        client
            .video_frame()
            .find_unique(
                video_frame::UniqueWhereParam::FileIdentifierTimestampEquals(
                    file_identifier.clone(),
                    frame_timestamp as i32,
                ),
            )
            .with(video_frame::caption::fetch())
            .exec()
            .await
    };

    match x {
        std::result::Result::Ok(Some(res)) => {
            let payload = SearchPayload::FrameCaption {
                id: res
                    .caption
                    .ok_or(anyhow::anyhow!("no caption record found"))?
                    .ok_or(anyhow::anyhow!("no caption record found"))?
                    .id as u64,
                file_identifier: file_identifier.clone(),
                timestamp: frame_timestamp,
            };
            save_text_embedding(
                &caption,
                payload,
                clip_model,
                qdrant,
                vector_db::DEFAULT_COLLECTION_NAME,
            )
            .await?;
        }
        std::result::Result::Ok(None) => {
            error!("failed to find frame caption");
        }
        Err(e) => {
            error!("failed to save frame caption embedding: {:?}", e);
        }
    }

    Ok(())
}
