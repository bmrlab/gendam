use super::payload::ContentIndexMetadata;
use content_base_context::ContentBaseCtx;
use content_base_task::{
    audio::trans_chunk_sum::{AudioTransChunkSumTask, AudioTransChunkSumTrait},
    image::description::ImageDescriptionTask,
    raw_text::chunk::{DocumentChunkTrait, RawTextChunkTask},
    video::trans_chunk_sum::VideoTransChunkSumTask,
    web_page::chunk::WebPageChunkTask,
    FileInfo,
};

/// Retrieves highlight text using metadata on the index
/// - Video: Summarized video content for the given timestamp range.
/// - Audio: Summarized audio content for the given timestamp range.
/// - Image: Retrieves the image description.
/// - RawText: Extracts text content for the given index range.
/// - WebPage: Extracts webpage content for the given index range.
pub async fn retrieve_highlight_text_with_metadata(
    ctx: &ContentBaseCtx,
    file_info: &FileInfo,
    metadata: &ContentIndexMetadata,
) -> anyhow::Result<String> {
    // let task_record = TaskRecord::from_content_base(file_info.file_identifier.as_str(), ctx).await;
    match metadata {
        ContentIndexMetadata::Video(video_metadata) => {
            let chunk_sum_task = VideoTransChunkSumTask;
            chunk_sum_task
                .sum_content(
                    file_info,
                    ctx,
                    video_metadata.start_timestamp,
                    video_metadata.end_timestamp,
                )
                .await
            // let chunk_task = VideoTransChunkTask;
            // let chunks = chunk_task.chunk_content(file_info, ctx).await?;
            // let matching_chunks = chunks
            //     .iter()
            //     .filter(|chunk| {
            //         chunk.start_timestamp <= video_metadata.start_timestamp
            //             && chunk.end_timestamp >= video_metadata.end_timestamp
            //     })
            //     .collect::<Vec<&Transcription>>();
            // tracing::info!("Matching chunks: {:?}", &matching_chunks);
            // let matching_chunk = matching_chunks.first();
            // if let Some(chunk) = matching_chunk {
            //     tracing::info!("Matching chunk: {:?}", &chunk.text);
            //     // Process the matching chunk hereq
            //     Ok(chunk.text.clone())
            // } else {
            //     tracing::warn!("No matching chunk found for the given metadata");
            //     Ok("".to_string())
            // }
        }
        ContentIndexMetadata::Audio(audio_metadata) => {
            let chunk_sum_task = AudioTransChunkSumTask;
            chunk_sum_task
                .sum_content(
                    file_info,
                    ctx,
                    audio_metadata.start_timestamp,
                    audio_metadata.end_timestamp,
                )
                .await
        }
        ContentIndexMetadata::Image(_) => {
            let chunk_sum_task = ImageDescriptionTask;
            chunk_sum_task.description_content(file_info, ctx).await
        }
        ContentIndexMetadata::RawText(text_metadata) => {
            let chunk_task = RawTextChunkTask;
            let chunks = chunk_task.chunk_content(file_info, ctx).await?;
            let content = chunks
                .iter()
                .skip(text_metadata.start_index)
                .take(text_metadata.end_index - text_metadata.start_index + 1)
                .map(|chunk| chunk.to_owned())
                .collect::<Vec<String>>()
                .join(" ");
            Ok(content)
        }
        ContentIndexMetadata::WebPage(webpage_metadata) => {
            let chunk_task = WebPageChunkTask;
            let chunks = chunk_task.chunk_content(file_info, ctx).await?;
            let content = chunks
                .iter()
                .skip(webpage_metadata.start_index)
                .take(webpage_metadata.end_index - webpage_metadata.start_index + 1)
                .map(|chunk| chunk.to_owned())
                .collect::<Vec<String>>()
                .join(" ");
            Ok(content)
        } // Add cases for other content types
          // _ => anyhow::bail!("Unsupported content type"),
    }
}
