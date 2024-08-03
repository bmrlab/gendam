use super::search::{retrieve_assets_for_search, SearchResultMetadata};
use crate::{ai::AIHandler, routes::{assets::types::FilePathWithAssetObjectData, tasks::types::ContentTaskTypeSpecta}};
use ai::llm::{LLMInferenceParams, LLMMessage};
use content_base::{
    audio::{
        transcript::{AudioTranscriptTask, AudioTranscriptTrait},
        AudioTaskType,
    },
    query::{payload::SearchMetadata, QueryPayload},
    video::{transcript::VideoTranscriptTask, VideoTaskType},
    ContentBase, ContentTaskType, FileInfo,
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
#[serde(rename_all = "camelCase")]
pub struct RetrievalResultPayload {
    pub file_path: FilePathWithAssetObjectData,
    pub metadata: SearchResultMetadata,
    pub score: f32,
    pub task_type: ContentTaskTypeSpecta,
}

#[derive(Serialize, Type)]
#[serde(tag = "result_type", content = "data")]
pub enum RAGResult {
    Reference(RetrievalResultPayload),
    Response(String),
    Error(String),
    Done,
}

pub async fn rag(
    library: &Library,
    content_base: &ContentBase,
    ai_handler: &AIHandler,
    input: RAGRequestPayload,
    tx: Sender<RAGResult>,
) -> anyhow::Result<()> {
    let retrieval_results = content_base
        .retrieve(QueryPayload::new(&input.query))
        .await?;
    let results = retrieve_assets_for_search(library, &retrieval_results, |item, file_path| {
        RetrievalResultPayload {
            file_path: file_path.clone().into(),
            metadata: SearchResultMetadata::from(&item.metadata),
            score: item.score,
            task_type: item.task_type.clone().into(),
        }
    })
    .await?;

    for ref_item in results.into_iter() {
        tx.send(RAGResult::Reference(ref_item)).await?;
    }

    let mut references_content = vec![];

    // find original chunk data, and use LLM to answer the question
    for ref_item in retrieval_results.iter() {
        match (&ref_item.metadata, &ref_item.task_type) {
            (
                SearchMetadata::Video(metadata),
                ContentTaskType::Video(VideoTaskType::TransChunkSumEmbed(_)),
            ) => {
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
                            if item.start_timestamp < metadata.start_timestamp {
                                continue;
                            }
                            if item.end_timestamp > metadata.end_timestamp {
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
            (
                SearchMetadata::Audio(metadata),
                ContentTaskType::Audio(AudioTaskType::TransChunkSumEmbed(_)),
            ) => {
                match AudioTranscriptTask
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
                            if item.start_timestamp < metadata.start_timestamp {
                                continue;
                            }
                            if item.end_timestamp > metadata.end_timestamp {
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
            _ => {
                // other combinations are considered invalid
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
