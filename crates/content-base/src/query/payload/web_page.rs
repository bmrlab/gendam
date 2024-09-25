use super::ContentIndexMetadata;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WebPageIndexMetadata {
    pub start_index: usize,
    pub end_index: usize,
}

impl TryFrom<ContentIndexMetadata> for WebPageIndexMetadata {
    type Error = anyhow::Error;

    fn try_from(metadata: ContentIndexMetadata) -> Result<Self, Self::Error> {
        match metadata {
            ContentIndexMetadata::WebPage(metadata) => Ok(metadata),
            _ => anyhow::bail!("metadata is not from raw text"),
        }
    }
}

impl From<WebPageIndexMetadata> for ContentIndexMetadata {
    fn from(metadata: WebPageIndexMetadata) -> Self {
        ContentIndexMetadata::WebPage(metadata)
    }
}

impl PartialEq for WebPageIndexMetadata {
    fn eq(&self, other: &Self) -> bool {
        self.start_index == other.start_index && self.end_index == other.end_index
    }
}

impl Eq for WebPageIndexMetadata {}

impl PartialOrd for WebPageIndexMetadata {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.start_index.partial_cmp(&other.start_index) {
            Some(std::cmp::Ordering::Equal) => self.end_index.partial_cmp(&other.end_index),
            other => other,
        }
    }
}

impl Ord for WebPageIndexMetadata {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}
