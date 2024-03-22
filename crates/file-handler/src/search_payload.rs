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
        id: u64,
        file_identifier: String,
        timestamp: i64,
    },
    FrameCaption {
        id: u64,
        file_identifier: String,
        timestamp: i64,
    },
    Transcript {
        id: u64,
        file_identifier: String,
        start_timestamp: i64,
        end_timestamp: i64,
    },
}

impl SearchPayload {
    pub fn get_id(&self) -> u64 {
        match self {
            SearchPayload::Frame { id, .. } => *id,
            SearchPayload::FrameCaption { id, .. } => *id,
            SearchPayload::Transcript { id, .. } => *id,
        }
    }

    pub fn get_uuid(&self) -> Uuid {
        Uuid::new_v5(&Uuid::NAMESPACE_OID, json!(self).to_string().as_bytes())
    }
}
