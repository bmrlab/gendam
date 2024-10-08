pub mod audio;
pub mod image;
pub mod raw_text;
pub mod video;
pub mod web_page;

use self::{
    audio::AudioIndexMetadata, image::ImageIndexMetadata, raw_text::RawTextIndexMetadata,
    video::VideoIndexMetadata, web_page::WebPageIndexMetadata,
};
use content_base_task::audio::trans_chunk_sum_embed::AudioTransChunkSumEmbedTask;
use content_base_task::audio::AudioTaskType;
use content_base_task::image::desc_embed::ImageDescEmbedTask;
use content_base_task::image::ImageTaskType;
use content_base_task::raw_text::chunk_sum_embed::RawTextChunkSumEmbedTask;
use content_base_task::raw_text::RawTextTaskType;
use content_base_task::video::trans_chunk_sum_embed::VideoTransChunkSumEmbedTask;
use content_base_task::video::VideoTaskType;
use content_base_task::web_page::chunk_sum_embed::WebPageChunkSumEmbedTask;
use content_base_task::web_page::WebPageTaskType;
use content_base_task::ContentTaskType;
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "content_type")]
pub enum ContentIndexMetadata {
    Video(VideoIndexMetadata),
    Audio(AudioIndexMetadata),
    Image(ImageIndexMetadata),
    RawText(RawTextIndexMetadata),
    WebPage(WebPageIndexMetadata),
}

impl From<&ContentIndexMetadata> for ContentTaskType {
    fn from(metadata: &ContentIndexMetadata) -> Self {
        match metadata {
            ContentIndexMetadata::Video(_) => ContentTaskType::Video(
                VideoTaskType::TransChunkSumEmbed(VideoTransChunkSumEmbedTask),
            ),
            ContentIndexMetadata::Audio(_) => ContentTaskType::Audio(
                AudioTaskType::TransChunkSumEmbed(AudioTransChunkSumEmbedTask),
            ),
            ContentIndexMetadata::Image(_) => {
                ContentTaskType::Image(ImageTaskType::DescEmbed(ImageDescEmbedTask))
            }
            ContentIndexMetadata::RawText(_) => {
                ContentTaskType::RawText(RawTextTaskType::ChunkSumEmbed(RawTextChunkSumEmbedTask))
            }
            ContentIndexMetadata::WebPage(_) => {
                ContentTaskType::WebPage(WebPageTaskType::ChunkSumEmbed(WebPageChunkSumEmbedTask))
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
/// SearchPayload is serialized and deserialized as the `payload` field of `qdrant_client::qdrant::SearchPoint`.
/// This process primarily occurs in:
/// 1. Serialization: When inserting or updating points in Qdrant (e.g., in the task_post_process function)
/// 2. Deserialization: When retrieving search results from Qdrant (e.g., in the group_results_by_asset function)
/// For video content, the serialized JSON format might look like this:
/// {
///   "file_identifier": "123456",
///   "task_type": "video-trans-chunk-sum-embed",
///   "metadata": {
///     "content_type": "Video",
///     "start_timestamp": 10000,
///     "end_timestamp": 15000
///   }
/// }
pub struct ContentIndexPayload {
    pub file_identifier: String,
    pub task_type: ContentTaskType,
    pub metadata: ContentIndexMetadata,
}

impl ContentIndexPayload {
    pub fn uuid(&self) -> Uuid {
        Uuid::new_v5(&Uuid::NAMESPACE_OID, json!(self).to_string().as_bytes())
    }

    pub fn file_identifier(&self) -> &str {
        &self.file_identifier
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResultData {
    pub file_identifier: String,
    pub score: f32,
    pub metadata: ContentIndexMetadata,
    pub highlight: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RetrievalResultData {
    pub file_identifier: String,
    pub task_type: ContentTaskType,
    pub score: f32,
    pub metadata: ContentIndexMetadata,
}

impl From<SearchResultData> for RetrievalResultData {
    fn from(data: SearchResultData) -> Self {
        Self {
            file_identifier: data.file_identifier.clone(),
            task_type: ContentTaskType::from(&data.metadata),
            score: data.score,
            metadata: data.metadata.clone(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchRequest {
    pub text: String,
}
