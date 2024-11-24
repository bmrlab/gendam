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
use payload::{
    audio::AudioSliceType, raw_text::RawTextChunkType, video::VideoSliceType, ContentIndexMetadata,
    ContentQueryResult,
};

const MAX_RETRIEVAL_COUNT: usize = 20;

pub struct ContentQueryPayload {
    pub query: String,
    pub max_count: Option<usize>,
    pub with_hit_reason: bool,
    pub with_reference_content: bool,
}

impl Default for ContentQueryPayload {
    fn default() -> Self {
        Self {
            query: String::new(),
            max_count: None,
            with_hit_reason: true,
            with_reference_content: true,
        }
    }
}

impl ContentBase {
    /// - 文本搜索流程
    ///     1. 获取全文搜索和向量搜索的结果（全文搜索和向量搜索只会搜索文本和图片）
    ///     2. 将上述结果进行 rank
    ///     3. 对上述 rank 的结果进行向上回溯
    ///     4. 填充 payload 信息
    #[tracing::instrument(err(Debug), skip_all, fields(query=%payload.query))]
    pub async fn query(
        &self,
        payload: ContentQueryPayload,
    ) -> anyhow::Result<Vec<ContentQueryResult>> {
        let search_model = self.query_payload_to_model(&payload).await?;
        let max_count = payload.max_count.unwrap_or(MAX_RETRIEVAL_COUNT);

        let query_results = self
            .surrealdb_client
            .try_read()?
            .search(search_model, true, max_count)
            .await?;

        // if payload.with_reference_content {
        //     for query_result in query_results.iter_mut() {
        //         let reference_content = self.reference_content(&query_result).await?;
        //         query_result.reference_content = Some(reference_content);
        //     }
        // }

        Ok(query_results)
    }

    async fn _reference_content(
        &self,
        query_result: &ContentQueryResult,
    ) -> anyhow::Result<String> {
        let ctx = self.ctx();
        let file_identifier = query_result.file_identifier.as_ref();
        let reference_content = match &query_result.metadata {
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

        Ok(reference_content)
    }
}
