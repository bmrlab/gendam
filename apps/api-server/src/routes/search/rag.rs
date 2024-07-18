use crate::ai::AIHandler;
use ai::llm::{LLMInferenceParams, LLMMessage};
use content_library::{Library, QdrantServerInfo};
use file_handler::{
    search::{handle_rag_retrieval, SearchResult},
    video::VideoHandler,
};
use serde::{Deserialize, Serialize};
use specta::Type;
use tokio::sync::mpsc::Sender;
use tracing::warn;

use super::search::{retrieve_assets_for_search, SearchResultPayload};

#[derive(Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct RAGRequestPayload {
    pub query: String,
}

#[derive(Serialize, Type)]
pub enum RAGResult {
    Reference(SearchResultPayload),
    Response(String),
    Error(String),
    Done,
}

pub async fn rag_with_video(
    library: &Library,
    qdrant_info: &QdrantServerInfo,
    ai_handler: &AIHandler,
    input: RAGRequestPayload,
    tx: Sender<RAGResult>,
) -> anyhow::Result<()> {
    let qdrant_client = library.qdrant_client();

    let references = handle_rag_retrieval(
        &input.query,
        qdrant_client,
        &qdrant_info.language_collection.name,
        &ai_handler.text_embedding,
    )
    .await?;

    let results = retrieve_assets_for_search(
        library,
        references
            .iter()
            .map(|v| SearchResult {
                file_identifier: v.file_identifier.clone(),
                start_timestamp: v.chunk_start_timestamp.clone(),
                end_timestamp: v.chunk_end_timestamp.clone(),
                score: v.score.clone(),
            })
            .collect(),
    )
    .await?;

    for ref_item in results.into_iter() {
        tx.send(RAGResult::Reference(ref_item)).await?;
    }

    // find original chunk data, and use LLM to answer the question
    let reference_content = references
        .iter()
        .filter_map(|ref_item| {
            let file_handler = VideoHandler::new(
                library.file_path(&ref_item.file_identifier),
                &ref_item.file_identifier,
                library.relative_artifacts_path(&ref_item.file_identifier),
                None,
            )
            .expect("no error when build video handler");
            match file_handler.get_transcript() {
                Ok(transcript) => Some(
                    transcript
                        .transcriptions
                        .into_iter()
                        // transcription is stored in order, so here just filter valid results
                        // TODO no need to iterate all transcriptions
                        .filter_map(|v| {
                            if (v.start_timestamp as i32) >= ref_item.chunk_start_timestamp
                                && (v.end_timestamp as i32) <= ref_item.chunk_end_timestamp
                            {
                                Some(v.text)
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>()
                        .join("\n"),
                ),
                _ => {
                    warn!("no transcript found for {}", ref_item.file_identifier);
                    None
                }
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

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
        reference_content, &input.query
    );

    let llm = ai_handler.llm.clone();
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
                break;
            }
            Err(e) => {
                tx.send(RAGResult::Error(e.to_string())).await?;
            }
        }
    }

    tx.send(RAGResult::Done).await?;

    Ok(())
}
