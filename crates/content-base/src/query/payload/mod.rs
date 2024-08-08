pub mod audio;
pub mod image;
pub mod raw_text;
pub mod video;
pub mod web_page;

use audio::AudioSearchMetadata;
use content_base_task::ContentTaskType;
use image::ImageSearchMetadata;
use qdrant_client::Payload;
use raw_text::RawTextSearchMetadata;
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;
use video::VideoSearchMetadata;
use web_page::WebPageSearchMetadata;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "content_type")]
pub enum SearchMetadata {
    Video(VideoSearchMetadata),
    Audio(AudioSearchMetadata),
    Image(ImageSearchMetadata),
    RawText(RawTextSearchMetadata),
    WebPage(WebPageSearchMetadata),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchPayload {
    pub file_identifier: String,
    pub task_type: ContentTaskType,
    pub metadata: SearchMetadata,
}

impl SearchPayload {
    pub fn uuid(&self) -> Uuid {
        Uuid::new_v5(&Uuid::NAMESPACE_OID, json!(self).to_string().as_bytes())
    }

    pub fn file_identifier(&self) -> &str {
        &self.file_identifier
    }
}

impl Into<Payload> for SearchPayload {
    fn into(self) -> Payload {
        json!(self)
            .try_into()
            .expect("json should be valid payload")
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct SearchResultData {
    pub file_identifier: String,
    pub score: f32,
    pub metadata: SearchMetadata,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct RetrievalResultData {
    pub file_identifier: String,
    pub task_type: ContentTaskType,
    pub score: f32,
    pub metadata: SearchMetadata,
}

pub trait SearchResult {
    fn file_identifier(&self) -> &str;
    fn metadata(&self) -> &SearchMetadata;
    fn score(&self) -> f32;
}

impl SearchResult for SearchResultData {
    fn file_identifier(&self) -> &str {
        &self.file_identifier
    }

    fn metadata(&self) -> &SearchMetadata {
        &self.metadata
    }

    fn score(&self) -> f32 {
        self.score
    }
}

impl SearchResult for RetrievalResultData {
    fn file_identifier(&self) -> &str {
        &self.file_identifier
    }

    fn metadata(&self) -> &SearchMetadata {
        &self.metadata
    }

    fn score(&self) -> f32 {
        self.score
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct SearchRequest {
    pub text: String,
}
