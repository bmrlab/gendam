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
    pub id: i32,
    pub file_identifier: String,
    pub frame_filename: String,
    pub timestamp: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FrameCaptionPayload {
    pub id: i32,
    pub file_identifier: String,
    pub frame_filename: String,
    pub caption: String,
    pub timestamp: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TranscriptPayload {
    pub id: i32,
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

    pub fn id(&self) -> i32 {
        match self {
            SearchPayload::Frame(frame) => frame.id,
            SearchPayload::FrameCaption(frame_caption) => frame_caption.id,
            SearchPayload::Transcript(transcript) => transcript.id,
        }
    }
}

// TODO this enum is not derived from SearchPayload
// maybe we can find some way to derive it not hard coding it
#[derive(Debug, Serialize, Deserialize)]
pub enum SearchRecordType {
    Frame,
    FrameCaption,
    Transcript,
}

impl SearchRecordType {
    pub fn as_str(&self) -> &str {
        match self {
            SearchRecordType::Frame => "Frame",
            SearchRecordType::FrameCaption => "FrameCaption",
            SearchRecordType::Transcript => "Transcript",
        }
    }
}
