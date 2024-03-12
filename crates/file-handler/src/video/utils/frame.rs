use ai::clip::CLIP;
use anyhow::{anyhow, Ok};
use prisma_lib::{video_frame, PrismaClient};
use qdrant_client::{
    client::QdrantClient, qdrant::PointStruct,
};
use serde_json::json;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::sync::RwLock;
use tracing::{debug, error};

use crate::search_payload::{FramePayload, SearchPayload};

pub async fn get_frame_content_embedding(
    file_identifier: String,
    client: Arc<PrismaClient>,
    frames_dir: impl AsRef<std::path::Path>,
    clip_model: Arc<RwLock<CLIP>>,
    qdrant: Arc<QdrantClient>,
) -> anyhow::Result<()> {
    let frame_paths = std::fs::read_dir(frames_dir.as_ref())?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, std::io::Error>>()?;

    // make sure collection have been crated
    super::make_sure_collection_created(
        qdrant.clone(),
        vector_db::VIDEO_FRAME_INDEX_NAME,
        clip_model.read().await.dim() as u64,
    ).await?;

    let mut join_set = tokio::task::JoinSet::new();

    for path in frame_paths {
        if path.extension() == Some(std::ffi::OsStr::new("png")) {
            debug!("handle file: {:?}", path);

            let clip_model = clip_model.clone();
            let client = client.clone();
            let qdrant = qdrant.clone();

            let file_name = path
                .file_name()
                .ok_or(anyhow!("invalid path"))?
                .to_str()
                .ok_or(anyhow!("invalid path"))?
                .to_owned();

            let frame_timestamp: i64 = file_name
                .split(".")
                .next()
                .unwrap_or("0")
                .parse()
                .unwrap_or(0);

            let file_identifier = file_identifier.clone();

            // FIXME 这里限制一下最大任务数量，因为出现过 axum 被 block 的情况
            if join_set.len() >= 3 {
                while let Some(_) = join_set.join_next().await {}
            }

            join_set.spawn(async move {
                // write data using prisma
                let x = {
                    client.video_frame().upsert(
                        video_frame::file_identifier_timestamp(
                            file_identifier.clone(),
                            frame_timestamp as i32,
                        ),
                        (file_identifier.clone(), frame_timestamp as i32, vec![]),
                        vec![],
                    ).exec().await
                    // drop the rwlock
                };

                match x {
                    std::result::Result::Ok(res) => {
                        let payload = SearchPayload::Frame(FramePayload {
                            id: res.id,
                            file_identifier: file_identifier.clone(),
                            frame_filename: file_name.to_string(),
                            timestamp: frame_timestamp,
                        });

                        let _ =
                            get_single_frame_content_embedding(payload, &path, clip_model, qdrant)
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

async fn get_single_frame_content_embedding(
    payload: SearchPayload,
    path: impl AsRef<std::path::Path>,
    clip_model: Arc<RwLock<ai::clip::CLIP>>,
    qdrant: Arc<QdrantClient>,
) -> anyhow::Result<()> {
    let embedding = clip_model
        .read()
        .await
        .get_image_embedding_from_file(path.as_ref())
        .await?;
    let embedding: Vec<f32> = embedding.iter().map(|&x| x).collect();

    let point = PointStruct::new(
        payload.uuid(),
        embedding.clone(),
        json!(payload)
            .try_into()
            .map_err(|_| anyhow::anyhow!("invalid payload"))?,
    );
    qdrant
        .upsert_points(vector_db::VIDEO_FRAME_INDEX_NAME, None, vec![point], None)
        .await?;

    // save into file to persist
    let embedding_path = path
        .as_ref()
        .to_str()
        .ok_or(anyhow!("invalid path"))?
        .replace(".png", ".embedding");
    let mut file = tokio::fs::File::create(embedding_path).await?;
    file.write_all(serde_json::to_string(&embedding)?.as_bytes())
        .await?;

    Ok(())
}
