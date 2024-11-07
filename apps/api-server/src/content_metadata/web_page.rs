use content_base::metadata::web_page::WebPageMetadata as RawWebPageMetadata;
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct WebPageMetadata {
    pub source_url: String,
}

impl From<&RawWebPageMetadata> for WebPageMetadata {
    fn from(metadata: &RawWebPageMetadata) -> Self {
        Self {
            source_url: metadata.source_url.clone(),
        }
    }
}
