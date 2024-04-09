use crate::search_payload::SearchPayload;
use ai::{text_embedding::TextEmbedding, BatchHandler};
use qdrant_client::{client::QdrantClient, qdrant::PointStruct};
use serde_json::json;
use std::sync::Arc;

pub(crate) mod caption;
pub(crate) mod clip;
pub(crate) mod frame;
pub(crate) mod transcript;

pub async fn save_text_embedding(
    text: &str,
    payload: SearchPayload,
    text_embedding: BatchHandler<TextEmbedding>,
    qdrant: Arc<QdrantClient>,
    collection_name: &str,
) -> anyhow::Result<()> {
    let embedding = text_embedding.process_single(text.to_string()).await?;
    let embedding: Vec<f32> = embedding.iter().map(|&x| x).collect();

    let point = PointStruct::new(
        payload.get_uuid().to_string(),
        embedding,
        json!(payload)
            .try_into()
            .map_err(|_| anyhow::anyhow!("invalid payload"))?,
    );
    qdrant
        .upsert_points(collection_name, None, vec![point], None)
        .await?;

    Ok(())
}

pub(self) fn get_frame_timestamp_from_path(
    path: impl AsRef<std::path::Path>,
) -> anyhow::Result<i64> {
    let file_name = path
        .as_ref()
        .file_name()
        .ok_or(anyhow::anyhow!("invalid path"))?
        .to_string_lossy()
        .to_string();

    let frame_timestamp: i64 = file_name
        .split(".")
        .next()
        .unwrap_or("0")
        .parse()
        .unwrap_or(0);

    Ok(frame_timestamp)
}
