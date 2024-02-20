use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum SearchRecordType {
    Frame,
    FrameCaption,
    Transcript,
}

impl SearchRecordType {
    pub fn index_name(&self) -> &str {
        match self {
            SearchRecordType::Frame => super::index::VIDEO_FRAME_INDEX_NAME,
            SearchRecordType::FrameCaption => super::index::VIDEO_FRAME_CAPTION_INDEX_NAME,
            SearchRecordType::Transcript => super::index::VIDEO_TRANSCRIPT_INDEX_NAME,
        }
    }
}
