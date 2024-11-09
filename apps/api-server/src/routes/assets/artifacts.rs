use crate::CtxWithLibrary;
use content_base_task::{
    audio::{trans_chunk_sum::AudioTransChunkSumTrait, transcript::AudioTranscriptTrait},
    image::description::ImageDescriptionTask,
    raw_text::{
        chunk::{DocumentChunkTrait, RawTextChunkTask},
        chunk_sum::{DocumentChunkSumTrait, RawTextChunkSumTask},
    },
    video::{trans_chunk_sum::VideoTransChunkSumTask, transcript::VideoTranscriptTask},
};
use rspc::{Router, RouterBuilder};
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Deserialize, Type, Debug)]
#[serde(rename_all = "camelCase")]
struct RawTextRequestPayload {
    hash: String,
    index: u32,
}

pub fn get_routes<TCtx>() -> RouterBuilder<TCtx>
where
    TCtx: CtxWithLibrary + Clone + Send + Sync + 'static,
{
    Router::<TCtx>::new()
        .query("video.transcript", |t| {
            #[derive(Deserialize, Type, Debug)]
            enum TranscriptType {
                Original,
                Summarization,
            }
            #[derive(Deserialize, Type, Debug)]
            #[serde(rename_all = "camelCase")]
            struct TranscriptRequestPayload {
                hash: String,
                #[specta(type = u32)]
                start_timestamp: i64,
                #[specta(type = u32)]
                end_timestamp: i64,
                request_type: TranscriptType,
            }
            #[derive(Serialize, Type, Debug)]
            #[serde(rename_all = "camelCase")]
            struct TranscriptResponse {
                content: String,
            }
            t({
                |ctx, input: TranscriptRequestPayload| async move {
                    let _library = ctx.library()?;
                    let content_base = ctx.content_base()?;

                    let content = {
                        match input.request_type {
                            TranscriptType::Original => match VideoTranscriptTask
                                .transcript_content(&input.hash, content_base.ctx())
                                .await
                            {
                                Ok(transcript) => {
                                    let mut transcript_vec = vec![];
                                    for item in transcript.transcriptions {
                                        if item.start_timestamp < input.start_timestamp {
                                            continue;
                                        }
                                        if item.end_timestamp > input.end_timestamp {
                                            break;
                                        }
                                        transcript_vec.push(item.text);
                                    }

                                    Ok(transcript_vec.join("\n"))
                                }
                                Err(e) => Err(e),
                            },
                            TranscriptType::Summarization => {
                                VideoTransChunkSumTask
                                    .sum_content(
                                        &input.hash,
                                        content_base.ctx(),
                                        input.start_timestamp,
                                        input.end_timestamp,
                                    )
                                    .await
                            }
                        }
                    };

                    let content = content.map_err(|e| {
                        rspc::Error::new(
                            rspc::ErrorCode::InternalServerError,
                            format!("failed to get transcript: {}", e),
                        )
                    })?;

                    Ok(TranscriptResponse { content })
                }
            })
        })
        .query("raw_text.chunk.content", |t| {
            t(|ctx, input: RawTextRequestPayload| async move {
                let _library = ctx.library()?;
                let content_base = ctx.content_base()?;

                match RawTextChunkTask
                    .chunk_content(&input.hash, content_base.ctx())
                    .await
                {
                    Ok(content) => {
                        if let Some(content) = content.get(input.index as usize) {
                            Ok(content.clone())
                        } else {
                            Err(rspc::Error::new(
                                rspc::ErrorCode::InternalServerError,
                                format!("failed to get raw text: {}", input.index),
                            ))
                        }
                    }
                    Err(e) => Err(rspc::Error::new(
                        rspc::ErrorCode::InternalServerError,
                        format!("failed to get raw text: {}", e),
                    )),
                }
            })
        })
        .query("raw_text.chunk.summarization", |t| {
            t(|ctx, input: RawTextRequestPayload| async move {
                let _library = ctx.library()?;
                let content_base = ctx.content_base()?;

                Ok(RawTextChunkSumTask
                    .sum_content(&input.hash, content_base.ctx(), input.index as usize)
                    .await
                    .map_err(|e| {
                        rspc::Error::new(
                            rspc::ErrorCode::InternalServerError,
                            format!("failed to get raw text: {}", e),
                        )
                    })?)
            })
        })
        .query("image.description", |t| {
            #[derive(Deserialize, Type, Debug)]
            #[serde(rename_all = "camelCase")]
            struct ImageRequestPayload {
                hash: String,
            }
            t(|ctx, input: ImageRequestPayload| async move {
                let _library = ctx.library()?;
                let content_base = ctx.content_base()?;

                Ok(ImageDescriptionTask
                    .description_content(&input.hash, content_base.ctx())
                    .await
                    .map_err(|e| {
                        rspc::Error::new(
                            rspc::ErrorCode::InternalServerError,
                            format!("failed to get raw text: {}", e),
                        )
                    })?)
            })
        })
}
