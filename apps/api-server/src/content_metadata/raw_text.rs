use content_base::metadata::raw_text::RawTextMetadata as RawRawTextMetadata;
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct RawTextMetadata {
    pub text_count: String,
}

impl From<&RawRawTextMetadata> for RawTextMetadata {
    fn from(metadata: &RawRawTextMetadata) -> Self {
        Self {
            text_count: metadata.text_count.to_string(),
        }
    }
}
