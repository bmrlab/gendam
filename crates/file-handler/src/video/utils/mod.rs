use crate::search_payload::SearchPayload;
use ai::{
    clip::{CLIPInput, CLIP},
    BatchHandler,
};
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
    clip: BatchHandler<CLIP>,
    qdrant: Arc<QdrantClient>,
    collection_name: &str,
) -> anyhow::Result<()> {
    let embedding = clip
        .process_single(CLIPInput::Text(text.to_string()))
        .await?;
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
