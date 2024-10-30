use crate::CtxWithLibrary;
use content_base::{
    audio::{trans_chunk_sum::AudioTransChunkSumTrait, transcript::AudioTranscriptTrait},
    image::description::ImageDescriptionTask,
    raw_text::{
        chunk::{DocumentChunkTrait, RawTextChunkTask},
        chunk_sum::{DocumentChunkSumTrait, RawTextChunkSumTask},
    },
    video::{trans_chunk_sum::VideoTransChunkSumTask, transcript::VideoTranscriptTask},
    FileInfo,
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
                start_timestamp: i32,
                end_timestamp: i32,
                request_type: TranscriptType,
            }
            #[derive(Serialize, Type, Debug)]
            #[serde(rename_all = "camelCase")]
            struct TranscriptResponse {
                content: String,
            }
            t({
                |ctx, input: TranscriptRequestPayload| async move {
                    let library = ctx.library()?;
                    let content_base = ctx.content_base()?;
                    let file_info = FileInfo {
                        file_identifier: input.hash.clone(),
                        file_path: library.absolute_file_path(&input.hash),
                    };

                    let content = {
                        match input.request_type {
                            TranscriptType::Original => match VideoTranscriptTask
                                .transcript_content(&file_info, content_base.ctx())
                                .await
                            {
                                Ok(transcript) => {
                                    let mut transcript_vec = vec![];
                                    for item in transcript.transcriptions {
                                        if item.start_timestamp < input.start_timestamp as i64 {
                                            continue;
                                        }
                                        if item.end_timestamp > input.end_timestamp as i64 {
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
                                        &file_info,
                                        content_base.ctx(),
                                        input.start_timestamp as i64,
                                        input.end_timestamp as i64,
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
                let library = ctx.library()?;
                let content_base = ctx.content_base()?;
                let file_info = FileInfo {
                    file_identifier: input.hash.clone(),
                    file_path: library.absolute_file_path(&input.hash),
                };

                match RawTextChunkTask
                    .chunk_content(&file_info, content_base.ctx())
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
                let library = ctx.library()?;
                let content_base = ctx.content_base()?;
                let file_info = FileInfo {
                    file_identifier: input.hash.clone(),
                    file_path: library.absolute_file_path(&input.hash),
                };

                Ok(RawTextChunkSumTask
                    .sum_content(&file_info, content_base.ctx(), input.index as usize)
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
                let library = ctx.library()?;
                let content_base = ctx.content_base()?;
                let file_info = FileInfo {
                    file_identifier: input.hash.clone(),
                    file_path: library.absolute_file_path(&input.hash),
                };

                Ok(ImageDescriptionTask
                    .description_content(&file_info, content_base.ctx())
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
