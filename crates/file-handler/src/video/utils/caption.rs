use super::save_text_embedding;
use anyhow::{anyhow, Ok};
use prisma_lib::{video_frame, video_frame_caption, PrismaClient};
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::sync::RwLock;
use tracing::{debug, error};
use vector_db::{FaissIndex, IndexInfo};

pub async fn get_frames_caption(
    frames_dir: impl AsRef<std::path::Path>,
    blip_model: Arc<RwLock<ai::blip::BLIP>>,
) -> anyhow::Result<()> {
    let frame_paths = std::fs::read_dir(frames_dir.as_ref())?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, std::io::Error>>()?;

    let mut join_set = tokio::task::JoinSet::new();

    for path in frame_paths {
        if path.extension() == Some(std::ffi::OsStr::new("png")) {
            debug!("get_frames_caption: {:?}", path);
            let blip_model = Arc::clone(&blip_model);

            join_set.spawn(async move {
                if let Err(e) = get_single_frame_caption(blip_model, path).await {
                    error!("failed to get frame caption: {:?}", e);
                }
            });
        }
    }

    while let Some(_) = join_set.join_next().await {}

    Ok(())
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

pub async fn get_frame_caption_embedding(
    file_identifier: String,
    client: Arc<RwLock<PrismaClient>>,
    frames_dir: impl AsRef<std::path::Path>,
    clip_model: Arc<RwLock<ai::clip::CLIP>>,
    embedding_index: FaissIndex,
    index_info: IndexInfo,
) -> anyhow::Result<()> {
    let frame_paths = std::fs::read_dir(&frames_dir)?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, std::io::Error>>()?;

    let mut join_set = tokio::task::JoinSet::new();

    for path in frame_paths {
        if path.extension() == Some(std::ffi::OsStr::new("caption")) {
            let clip_model = Arc::clone(&clip_model);
            let file_identifier = file_identifier.clone();
            let client = client.clone();
            let embedding_index = embedding_index.clone();
            let index_info = index_info.clone();

            // FIXME 这里限制一下最大任务数量，因为出现过 axum 被 block 的情况
            if join_set.len() >= 3 {
                while let Some(_) = join_set.join_next().await {}
            }

            join_set.spawn(async move {
                if let Err(e) = get_single_frame_caption_embedding(
                    file_identifier,
                    client,
                    path,
                    clip_model,
                    embedding_index,
                    index_info,
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
    client: Arc<RwLock<PrismaClient>>,
    path: impl AsRef<std::path::Path>,
    clip_model: Arc<RwLock<ai::clip::CLIP>>,
    embedding_index: FaissIndex,
    index_info: IndexInfo,
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
            save_text_embedding(
                &caption,
                res.id as u64,
                clip_model,
                embedding_index,
                index_info,
            )
            .await?;
        }
        Err(e) => {
            error!("failed to save frame caption embedding: {:?}", e);
        }
    }

    Ok(())
}
