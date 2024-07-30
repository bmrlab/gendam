use super::search::{retrieve_assets_for_search, SearchResultPayload};
use crate::ai::AIHandler;
use ai::llm::{LLMInferenceParams, LLMMessage};
use content_base::{
    audio::transcript::AudioTranscriptTrait,
    query::{RAGPayload, SearchResult},
    video::transcript::VideoTranscriptTask,
    ContentBase, FileInfo,
};
use content_library::Library;
use serde::{Deserialize, Serialize};
use specta::Type;
use tokio::sync::mpsc::Sender;

#[derive(Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct RAGRequestPayload {
    pub query: String,
}

#[derive(Serialize, Type)]
#[serde(tag = "result_type", content = "data")]
pub enum RAGResult {
    Reference(SearchResultPayload),
    Response(String),
    Error(String),
    Done,
}

pub async fn rag_with_video(
    library: &Library,
    content_base: &ContentBase,
    ai_handler: &AIHandler,
    input: RAGRequestPayload,
    tx: Sender<RAGResult>,
) -> anyhow::Result<()> {
    let payload = RAGPayload::new(&input.query);
    let references = content_base.retrieval(payload).await?;

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

    let mut references_content = vec![];

    // find original chunk data, and use LLM to answer the question
    for ref_item in references.iter() {
        match VideoTranscriptTask
            .transcript_content(
                &FileInfo {
                    file_identifier: ref_item.file_identifier.clone(),
                    file_path: library.file_path(&ref_item.file_identifier),
                },
                content_base.ctx(),
            )
            .await
        {
            Ok(transcript) => {
                let mut transcript_vec = vec![];
                for item in transcript.transcriptions {
                    if (item.start_timestamp as i32) < ref_item.chunk_start_timestamp {
                        continue;
                    }
                    if (item.end_timestamp as i32) > ref_item.chunk_end_timestamp {
                        break;
                    }
                    transcript_vec.push(item.text);
                }

                references_content.push(transcript_vec.join("\n"));
            }
            _ => {
                tracing::warn!("failed to get transcript for {}", ref_item.file_identifier);
            }
        }
    }

    let reference_content = references_content.join("\n");

    let system_prompt = r#"You are an assistant good at answer questions according to pieces of video transcript.
You should try to answer user question according to the provided video transcripts.
Keep your answer ground in the facts of the DOCUMENT.
Try to response in markdown, with proper title, subtitles and bullet points.
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
        .0
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
