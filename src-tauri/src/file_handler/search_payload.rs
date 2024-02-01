use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub enum SearchPayload {
    Frame(FramePayload),
    FrameCaption(FrameCaptionPayload),
    Transcript(TranscriptPayload),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FramePayload {
    pub file_identifier: String,
    pub frame_filename: String,
    pub timestamp: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FrameCaptionPayload {
    pub file_identifier: String,
    pub frame_filename: String,
    pub caption: String,
    pub timestamp: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TranscriptPayload {
    pub file_identifier: String,
    pub transcript: String,
    pub start_timestamp: i64,
    pub end_timestamp: i64,
}

impl SearchPayload {
    pub fn uuid(&self) -> String {
        let json_string = json!(self).to_string();
        let uuid = Uuid::new_v5(&Uuid::NAMESPACE_OID, json_string.as_bytes());
        uuid.to_string()
    }
}
