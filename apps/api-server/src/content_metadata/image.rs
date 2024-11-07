use content_base::metadata::image::ImageMetadata as RawImageMetadata;
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ImageMetadata {
    pub width: u32,
    pub height: u32,
    pub color: String,
}

impl From<&RawImageMetadata> for ImageMetadata {
    fn from(metadata: &RawImageMetadata) -> Self {
        Self {
            width: metadata.width,
            height: metadata.height,
            color: metadata.color.to_string(),
        }
    }
}
