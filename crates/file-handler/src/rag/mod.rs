use crate::{search::payload::SearchPayload, SearchRecordType};
use ai::{
    llm::{LLMInferenceParams, LLMMessage},
    LLMModel, TextEmbeddingModel,
};
use qdrant_client::{
    client::QdrantClient,
    qdrant::{Condition, Filter, SearchPoints},
};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;

#[derive(Clone, Debug)]
pub struct RAGReference {
    pub file_identifier: String,
    pub start_timestamp: i32,
    pub end_timestamp: i32,
    pub score: f32,
}

#[derive(Clone, Debug)]
pub enum RAGResult {
    Reference(RAGReference),
    Response(String),
    Error(String),
    Done,
}

pub async fn handle_rag(
    query: &str,
    qdrant: Arc<QdrantClient>,
    language_collection_name: &str,
    text_embedding: &TextEmbeddingModel,
    llm: &LLMModel,
    tx: Sender<RAGResult>,
) -> anyhow::Result<()> {
    let text_embedding = text_embedding.process_single(query.to_string()).await?;

    let search_point = SearchPoints {
        collection_name: language_collection_name.into(),
        vector: text_embedding,
        limit: 5,
        with_payload: Some(true.into()),
        filter: Some(Filter::all(vec![Condition::matches(
            "record_type",
            SearchRecordType::TranscriptChunkSummarization.to_string(),
        )])),
        ..Default::default()
    };

    let response = qdrant.search_points(&search_point).await?;
    let scored_points = response.result;

    let mut original_transcripts = vec![];

    let reference: Vec<RAGReference> = scored_points
        .iter()
        .filter_map(|v| {
            let payload = serde_json::from_value::<SearchPayload>(json!(v.payload));
            if let Ok(SearchPayload::TranscriptChunkSummarization {
                file_identifier,
                start_timestamp,
                end_timestamp,
                chunk_content,
            }) = payload
            {
                original_transcripts.push(chunk_content);
                Some(RAGReference {
                    file_identifier: file_identifier.to_string(),
                    start_timestamp: start_timestamp as i32,
                    end_timestamp: end_timestamp as i32,
                    score: v.score,
                })
            } else {
                None
            }
        })
        .collect();

    for ref_item in reference {
        tx.send(RAGResult::Reference(ref_item)).await?;
    }

    let system_prompt = r#"You are an assistant good at answer questions according to pieces of video transcript.
You should try to answer user question according to the provided video transcripts.
Keep your answer ground in the facts of the DOCUMENT.
Try to response in a structured format like markdown, with proper title, subtitles and bullet points.
If the DOCUMENT doesn't contain the facts to answer the QUESTION return {I don't know} in the question's language.
"#;
    let user_prompt = format!(
        r#"TRANSCRIPTS:
{}

QUESTION:
{}
"#,
        original_transcripts.join("\n"),
        query
    );

    let mut response = llm
        .process_single((
            vec![
                LLMMessage::System(system_prompt.into()),
                LLMMessage::User(user_prompt.into()),
            ],
            LLMInferenceParams::default(),
        ))
        .await?;

    while let Some(content) = response.next().await {
        match content {
            Ok(Some(data)) => {
                tx.send(RAGResult::Response(data)).await?;
            }
            Ok(None) => {
                tx.send(RAGResult::Done).await?;
            }
            Err(e) => {
                tx.send(RAGResult::Error(e.to_string())).await?;
            }
        }
    }

    Ok(())
}
