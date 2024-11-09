mod data_handler;
pub mod model;
pub mod payload;

use crate::ContentBase;
use content_base_task::{
    audio::transcript::{AudioTranscriptTask, AudioTranscriptTrait},
    image::description::ImageDescriptionTask,
    raw_text::chunk::{DocumentChunkTrait, RawTextChunkTask},
    video::{frame_description::VideoFrameDescriptionTask, transcript::VideoTranscriptTask},
};
use model::hit_result::HitResult;
use payload::{
    audio::AudioSliceType, raw_text::RawTextChunkType, video::VideoSliceType, ContentIndexMetadata,
    RetrievalResultData, SearchResultData,
};

const RETRIEVAL_COUNT: u64 = 20;
const MAX_RANK_COUNT: usize = 10;

pub struct QueryPayload {
    query: String,
}

impl QueryPayload {
    pub fn new(query: &str) -> Self {
        Self {
            query: query.to_string(),
        }
    }
}

impl ContentBase {
    /// - 文本搜索流程
    ///     1. 获取全文搜索和向量搜索的结果（全文搜索和向量搜索只会搜索文本和图片）
    ///     2. 将上述结果进行 rank
    ///     3. 对上述 rank 的结果进行向上回溯
    ///     4. 填充 payload 信息
    pub async fn query(
        &self,
        payload: QueryPayload,
        max_count: Option<usize>,
    ) -> anyhow::Result<Vec<SearchResultData>> {
        let max_count = max_count.unwrap_or(MAX_RANK_COUNT);
        let with_highlight = true;

        let hit_result = self
            .db
            .try_read()?
            .search(
                self.query_payload_to_model(payload).await?,
                with_highlight,
                max_count,
            )
            .await?;

        Ok(hit_result
            .into_iter()
            .filter_map(|hit| self.expand_hit_result(hit).ok())
            .flatten()
            .collect::<Vec<SearchResultData>>())
    }

    pub async fn retrieve(
        &self,
        payload: QueryPayload,
    ) -> anyhow::Result<Vec<RetrievalResultData>> {
        let search_results = self.query(payload, Some(RETRIEVAL_COUNT as usize)).await?;
        let mut retrieval_results: Vec<RetrievalResultData> = vec![];
        let ctx = self.ctx();
        for search_result in search_results.into_iter() {
            let file_identifier = search_result.file_identifier.as_ref();
            let reference_content = match &search_result.metadata {
                ContentIndexMetadata::Video(metadata) => match metadata.slice_type {
                    VideoSliceType::Visual => {
                        let frame_description = VideoFrameDescriptionTask
                            .frame_description_content(
                                file_identifier,
                                ctx,
                                metadata.start_timestamp,
                                metadata.end_timestamp,
                            )
                            .await?;
                        frame_description
                    }
                    VideoSliceType::Audio => {
                        // TODO: 应该取一个区间的 transcript
                        let transcript = VideoTranscriptTask
                            .transcript_content(file_identifier, ctx)
                            .await?;
                        let transcript_vec = transcript
                            .transcriptions
                            .iter()
                            .filter(|item| {
                                item.start_timestamp >= metadata.start_timestamp
                                    && item.end_timestamp <= metadata.end_timestamp
                            })
                            .map(|item| item.text.clone())
                            .collect::<Vec<String>>();
                        transcript_vec.join("\n")
                    }
                },
                ContentIndexMetadata::Audio(metadata) => match metadata.slice_type {
                    AudioSliceType::Transcript => {
                        let transcript = AudioTranscriptTask
                            .transcript_content(file_identifier, ctx)
                            .await?;
                        let transcript_vec = transcript
                            .transcriptions
                            .iter()
                            .filter(|item| {
                                item.start_timestamp >= metadata.start_timestamp
                                    && item.end_timestamp <= metadata.end_timestamp
                            })
                            .map(|item| item.text.clone())
                            .collect::<Vec<String>>();
                        transcript_vec.join("\n")
                    }
                },
                ContentIndexMetadata::RawText(metadata) => match metadata.chunk_type {
                    RawTextChunkType::Content => {
                        let chunks = RawTextChunkTask.chunk_content(file_identifier, ctx).await?;
                        let raw_text_vec = chunks
                            [metadata.start_index as usize..=metadata.end_index as usize]
                            .iter()
                            .cloned()
                            .collect::<Vec<String>>();
                        raw_text_vec.join("\n")
                    }
                },
                ContentIndexMetadata::Image(_) => {
                    let description = ImageDescriptionTask
                        .description_content(file_identifier, ctx)
                        .await?;
                    description
                }
                _ => "".to_string(),
            };
            retrieval_results.push(RetrievalResultData {
                file_identifier: search_result.file_identifier,
                metadata: search_result.metadata,
                score: search_result.score,
                reference_content,
            });
        }
        Ok(retrieval_results)
    }
}
