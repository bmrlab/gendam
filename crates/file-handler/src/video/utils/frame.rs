use ai::clip::CLIP;
use anyhow::{anyhow, Ok};
use prisma_lib::{video_frame, PrismaClient};
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::sync::RwLock;
use tracing::{debug, error};
use vector_db::{FaissIndex, IndexInfo};

pub async fn get_frame_content_embedding(
    file_identifier: String,
    client: Arc<RwLock<PrismaClient>>,
    frames_dir: impl AsRef<std::path::Path>,
    clip_model: Arc<RwLock<CLIP>>,
    embedding_index: FaissIndex,
    index_info: IndexInfo,
) -> anyhow::Result<()> {
    let frame_paths = std::fs::read_dir(frames_dir.as_ref())?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, std::io::Error>>()?;

    let mut join_set = tokio::task::JoinSet::new();

    for path in frame_paths {
        if path.extension() == Some(std::ffi::OsStr::new("png")) {
            debug!("handle file: {:?}", path);

            let clip_model = clip_model.clone();
            let embedding_index = embedding_index.clone();
            let client = client.clone();
            let index_info = index_info.clone();

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

            let file_identifier = file_identifier.clone();
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
                            index_info,
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

async fn get_single_frame_content_embedding(
    id: u64,
    path: impl AsRef<std::path::Path>,
    clip_model: Arc<RwLock<ai::clip::CLIP>>,
    embedding_index: FaissIndex,
    index_info: IndexInfo,
) -> anyhow::Result<()> {
    let embedding = clip_model
        .read()
        .await
        .get_image_embedding_from_file(path.as_ref())
        .await?;
    let embedding: Vec<f32> = embedding.iter().map(|&x| x).collect();

    embedding_index
        .add(id, embedding.clone(), index_info)
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
