use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct SearchPayload {
    // add more filed if sophisticated filtering is needed
    pub id: u64,
    pub file_identifier: String,
    pub start_timestamp: i64,
    pub end_timestamp: i64,
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
