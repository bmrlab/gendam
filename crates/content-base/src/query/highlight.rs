use super::payload::ContentIndexMetadata;
use content_base_context::ContentBaseCtx;
use content_base_task::{
    audio::trans_chunk::{AudioTransChunkTask, AudioTranscriptChunkTrait},
    image::description::ImageDescriptionTask,
    raw_text::{
        chunk::DocumentChunkTrait,
        chunk_sum::{DocumentChunkSumTrait, RawTextChunkSumTask},
    },
    video::trans_chunk::VideoTransChunkTask,
    web_page::chunk::WebPageChunkTask,
    FileInfo,
};

/// Retrieves highlight text using metadata of content's index in vector store
/// - Video: Summarized video content for the given timestamp range.
/// - Audio: Summarized audio content for the given timestamp range.
/// - Image: Retrieves the image description.
/// - RawText: Extracts text content for the given index range.
/// - WebPage: Extracts webpage content for the given index range.
pub(super) async fn retrieve_highlight(
    ctx: &ContentBaseCtx,
    file_identifier: &str,
    metadata: &ContentIndexMetadata,
) -> Option<String> {
    let file_info = FileInfo {
        file_identifier: file_identifier.to_string(),
        file_path: "/-/invalid/-/".to_string().into(),
    };
    // let task_record = TaskRecord::from_content_base(file_info.file_identifier.as_str(), ctx).await;
    match metadata {
        ContentIndexMetadata::Video(video_metadata) => {
            // 这里不好用 VideoTransChunkSumTask 而要用 VideoTransChunkTask
            // sum_content 方法会精确寻找 {start_timestamp} - {end_timestamp} 的片段，但是这里的 metadata 是 merge 过的
            // 会导致找不到对应的片段
            // let chunk_sum_task = VideoTransChunkSumTask;
            // chunk_sum_task.sum_content(&file_info, ctx, video_metadata.start_timestamp, video_metadata.end_timestamp).await.ok()
            let chunk_task = VideoTransChunkTask;
            match chunk_task.chunk_content(&file_info, ctx).await {
                Ok(chunks) => {
                    let matching_chunks = chunks
                        .iter()
                        .filter(|chunk| {
                            chunk.start_timestamp >= video_metadata.start_timestamp
                                && chunk.end_timestamp <= video_metadata.end_timestamp
                        })
                        .map(|chunk| chunk.text.clone())
                        .collect::<Vec<_>>()
                        .join(" ");
                    Some(matching_chunks)
                }
                Err(e) => {
                    tracing::error!("Failed to retrieve video chunks: {:?}", e);
                    None
                }
            }
        }
        ContentIndexMetadata::Audio(audio_metadata) => {
            // 同 Video，要用 AudioTransChunkTask
            // let chunk_sum_task = AudioTransChunkSumTask;
            // chunk_sum_task.sum_content(&file_info, ctx, audio_metadata.start_timestamp, audio_metadata.end_timestamp).await.ok()
            let chunk_task = AudioTransChunkTask;
            match chunk_task.chunk_content(&file_info, ctx).await {
                Ok(chunks) => {
                    let matching_chunks = chunks
                        .iter()
                        .filter(|chunk| {
                            chunk.start_timestamp >= audio_metadata.start_timestamp
                                && chunk.end_timestamp <= audio_metadata.end_timestamp
                        })
                        .map(|chunk| chunk.text.clone())
                        .collect::<Vec<_>>()
                        .join(" ");
                    Some(matching_chunks)
                }
                Err(e) => {
                    tracing::error!("Failed to retrieve video chunks: {:?}", e);
                    None
                }
            }
        }
        ContentIndexMetadata::Image(_) => {
            let chunk_sum_task = ImageDescriptionTask;
            chunk_sum_task
                .description_content(&file_info, ctx)
                .await
                .ok()
        }
        ContentIndexMetadata::RawText(text_metadata) => {
            // 这里不能用 RawTextChunkTask 要用 RawTextChunkSumTask
            // 每个 summary 使用了 chunk 的前后片段，有时候 start_index - end_index 可能是空的，但 summary 不一定是空的
            // let chunk_task = RawTextChunkTask;
            // match chunk_task.chunk_content(&file_info, ctx).await {
            //     Ok(chunks) => {
            //         let content = chunks.iter()
            //             .skip(text_metadata.start_index)
            //             .take(text_metadata.end_index - text_metadata.start_index + 1)
            //             .map(|chunk| chunk.to_owned()).collect::<Vec<String>>().join(" ");
            // TODO: 一种更好的实现还是用 RawTextChunkTask，但是要多取 start_index 之前的一个片段和 end_index 之后的一个片段
            let chunk_sum_task = RawTextChunkSumTask;
            chunk_sum_task
                .sum_content(&file_info, ctx, text_metadata.start_index)
                .await
                .ok()
        }
        ContentIndexMetadata::WebPage(webpage_metadata) => {
            let chunk_task = WebPageChunkTask;
            match chunk_task.chunk_content(&file_info, ctx).await {
                Ok(chunks) => {
                    let content = chunks
                        .iter()
                        .skip(webpage_metadata.start_index)
                        .take(webpage_metadata.end_index - webpage_metadata.start_index + 1)
                        .map(|chunk| chunk.to_owned())
                        .collect::<Vec<String>>()
                        .join(" ");
                    Some(content)
                }
                Err(_) => None,
            }
        } // Add cases for other content types
          // _ => anyhow::bail!("Unsupported content type"),
    }
}
