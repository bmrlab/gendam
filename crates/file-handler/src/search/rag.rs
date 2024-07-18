use super::payload::SearchPayload;
use crate::SearchRecordType;
use ai::TextEmbeddingModel;
use qdrant_client::{
    qdrant::{Condition, Filter, SearchPointsBuilder},
    Qdrant,
};
use serde_json::json;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct RAGReference {
    pub file_identifier: String,
    pub chunk_start_timestamp: i32,
    pub chunk_end_timestamp: i32,
    pub score: f32,
}

pub async fn handle_rag_retrieval(
    query: &str,
    qdrant: Arc<Qdrant>,
    language_collection_name: &str,
    text_embedding: &TextEmbeddingModel,
) -> anyhow::Result<Vec<RAGReference>> {
    let text_embedding = text_embedding.process_single(query.to_string()).await?;

    let response = qdrant
        .search_points(
            SearchPointsBuilder::new(language_collection_name, text_embedding, 5)
                .filter(Filter::all(vec![Condition::matches(
                    "record_type",
                    SearchRecordType::TranscriptChunkSummarization.to_string(),
                )]))
                .with_payload(true),
        )
        .await?;
    let scored_points = response.result;

    Ok(scored_points
        .into_iter()
        .filter_map(|v| {
            let payload = serde_json::from_value::<SearchPayload>(json!(v.payload));
            if let Ok(SearchPayload::TranscriptChunkSummarization {
                file_identifier,
                start_timestamp,
                end_timestamp,
            }) = payload
            {
                Some(RAGReference {
                    file_identifier: file_identifier.to_string(),
                    chunk_start_timestamp: start_timestamp as i32,
                    chunk_end_timestamp: end_timestamp as i32,
                    score: v.score,
                })
            } else {
                None
            }
        })
        .collect())
}
