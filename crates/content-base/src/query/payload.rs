use qdrant_client::Payload;
use serde::{Deserialize, Serialize};
use serde_json::json;
use strum_macros::EnumDiscriminants;
use uuid::Uuid;

#[derive(Serialize, Deserialize, EnumDiscriminants)]
#[strum_discriminants(derive(Serialize, Deserialize, strum_macros::Display))]
#[strum_discriminants(name(SearchRecordType))]
#[serde(tag = "record_type")]
pub enum SearchPayload {
    Frame {
        file_identifier: String,
        timestamp: i64,
    },
    FrameCaption {
        file_identifier: String,
        timestamp: i64,
        method: String,
    },
    Transcript {
        file_identifier: String,
        start_timestamp: i64,
        end_timestamp: i64,
        method: String,
    },
    TranscriptChunk {
        file_identifier: String,
        start_timestamp: i64,
        end_timestamp: i64,
    },
    TranscriptChunkSummarization {
        file_identifier: String,
        start_timestamp: i64,
        end_timestamp: i64,
    },
}

impl SearchPayload {
    pub fn get_uuid(&self) -> Uuid {
        Uuid::new_v5(&Uuid::NAMESPACE_OID, json!(self).to_string().as_bytes())
    }

    pub fn get_file_identifier(&self) -> &str {
        match self {
            SearchPayload::Frame {
                file_identifier, ..
            } => file_identifier,
            SearchPayload::FrameCaption {
                file_identifier, ..
            } => file_identifier,
            SearchPayload::Transcript {
                file_identifier, ..
            } => file_identifier,
            SearchPayload::TranscriptChunk {
                file_identifier, ..
            } => file_identifier,
            SearchPayload::TranscriptChunkSummarization {
                file_identifier, ..
            } => file_identifier,
        }
    }
}

impl Into<Payload> for SearchPayload {
    fn into(self) -> Payload {
        json!(self)
            .try_into()
            .expect("json should be valid payload")
    }
}

#[derive(Clone, Debug)]
pub struct VideoRAGReference {
    pub file_identifier: String,
    pub chunk_start_timestamp: i32,
    pub chunk_end_timestamp: i32,
    pub score: f32,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct RetrievalResult {
    pub file_identifier: String,
    pub timestamp: i32,
    pub record_type: SearchRecordType,
    pub score: f32,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct AudioSearchResultMetadata {
    pub start_timestamp: i32,
    pub end_timestamp: i32,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct VideoSearchResultMetadata {
    pub start_timestamp: i32,
    pub end_timestamp: i32,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum SearchResultMetadata {
    Audio(AudioSearchResultMetadata),
    Video(VideoSearchResultMetadata),
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct SearchResult {
    pub file_identifier: String,
    pub score: f32,
    pub metadata: SearchResultMetadata,
}

impl From<&VideoRAGReference> for SearchResult {
    fn from(reference: &VideoRAGReference) -> Self {
        Self {
            file_identifier: reference.file_identifier.clone(),
            score: reference.score,
            metadata: SearchResultMetadata::Video(VideoSearchResultMetadata {
                start_timestamp: reference.chunk_start_timestamp as i32,
                end_timestamp: reference.chunk_end_timestamp as i32,
            }),
        }
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct SearchRequest {
    pub text: String,
    pub record_type: Option<Vec<SearchRecordType>>,
}

pub enum SearchType {
    Frame,
    FrameCaption,
    Transcript,
}

pub(crate) struct ClipRetrievalInfo {
    pub file_identifier: String,
    pub start_timestamp: i64,
    pub end_timestamp: i64,
    pub scores: Vec<f32>,
}
