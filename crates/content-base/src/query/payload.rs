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
