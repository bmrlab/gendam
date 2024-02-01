use ndarray::Axis;
use qdrant_client::{client::QdrantClientConfig, qdrant::SearchPoints};
use serde_json::json;

use self::search_payload::SearchPayload;

pub(self) mod audio;
pub mod embedding;
pub mod search_payload;
pub mod video;

pub const QDRANT_COLLECTION_NAME: &str = "muse-v2";
pub const EMBEDDING_DIM: u64 = 512;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct SearchResult {
    pub payload: search_payload::SearchPayload,
    pub score: f32,
}

pub enum SearchType {
    Frame,
    FrameCaption,
    Transcript,
}

pub async fn handle_search(
    text: &str,
    resources_dir: impl AsRef<std::path::Path>,
    skip: Option<u64>,
    limit: Option<u64>,
) -> anyhow::Result<Vec<SearchResult>> {
    let clip_model = embedding::clip::CLIP::new(
        resources_dir.as_ref().join("visual.onnx"),
        resources_dir.as_ref().join("textual.onnx"),
        resources_dir.as_ref().join("tokenizer.json"),
    )?;

    let client = QdrantClientConfig::from_url("http://0.0.0.0:6334").build()?;

    let embedding = clip_model.get_text_embedding(&text).await?;
    let embedding: Vec<f32> = embedding
        .index_axis(Axis(0), 0)
        .iter()
        .map(|&x| x)
        .collect();
    let search_result = client
        .search_points(&SearchPoints {
            collection_name: QDRANT_COLLECTION_NAME.into(),
            vector: embedding,
            limit: limit.unwrap_or(10),
            offset: skip,
            with_payload: Some(true.into()),
            ..Default::default()
        })
        .await?;

    Ok(search_result
        .result
        .iter()
        .map(|value| {
            let payload = serde_json::from_value::<SearchPayload>(json!(value.payload));
            match payload {
                Ok(payload) => Some(SearchResult {
                    payload,
                    score: value.score,
                }),
                _ => None,
            }
        })
        .filter_map(|x| x)
        .collect())
}
