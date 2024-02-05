use ndarray::Axis;
use qdrant_client::{
    client::QdrantClientConfig,
    qdrant::{Condition, Filter, SearchPoints},
};
use serde_json::json;

use self::search_payload::{SearchPayload, SearchRecordType};

pub(self) mod audio;
pub mod embedding;
pub mod search_payload;
pub mod video;

// TODO constants should be extracted into global config
pub const QDRANT_COLLECTION_NAME: &str = "muse-v2";
pub const EMBEDDING_DIM: u64 = 512;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct SearchResult {
    pub payload: search_payload::SearchPayload,
    pub score: f32,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct SearchRequest {
    pub text: String,
    pub record_type: Option<SearchRecordType>,
    pub skip: Option<u64>,
    pub limit: Option<u64>,
}

pub enum SearchType {
    Frame,
    FrameCaption,
    Transcript,
}

pub async fn handle_search(
    payload: SearchRequest,
    resources_dir: impl AsRef<std::path::Path>,
) -> anyhow::Result<Vec<SearchResult>> {
    let clip_model =
        embedding::clip::CLIP::new(embedding::clip::model::CLIPModel::ViTB32, &resources_dir)
            .await?;

    let client = QdrantClientConfig::from_url("http://0.0.0.0:6334").build()?;

    let embedding = clip_model.get_text_embedding(&payload.text).await?;
    let embedding: Vec<f32> = embedding
        .index_axis(Axis(0), 0)
        .iter()
        .map(|&x| x)
        .collect();
    let filter = if let Some(record_type) = payload.record_type {
        Some(Filter::must_not([Condition::is_empty(record_type.as_str())]))
    } else {
        None
    };
    let search_result = client
        .search_points(&SearchPoints {
            collection_name: QDRANT_COLLECTION_NAME.into(),
            vector: embedding,
            limit: payload.limit.unwrap_or(10),
            offset: payload.skip,
            with_payload: Some(true.into()),
            filter,
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
