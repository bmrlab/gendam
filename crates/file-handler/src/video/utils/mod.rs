use crate::search::payload::SearchPayload;
use ai::AsTextEmbeddingModel;
use qdrant_client::{
    client::QdrantClient,
    qdrant::{point_id::PointIdOptions, PointId, PointStruct},
};
use serde_json::json;
use std::sync::Arc;

pub(crate) mod caption;
pub(crate) mod clip;
pub(crate) mod frame;
pub(crate) mod transcript;

pub async fn save_text_embedding(
    text: &str,
    payload: SearchPayload,
    text_embedding: &dyn AsTextEmbeddingModel,
    qdrant: Arc<QdrantClient>,
    collection_name: &str,
) -> anyhow::Result<()> {
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
            return Ok(());
        }
        _ => {}
    }

    let embedding = text_embedding
        .get_texts_embedding_tx()
        .process_single(text.to_string())
        .await?;

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
