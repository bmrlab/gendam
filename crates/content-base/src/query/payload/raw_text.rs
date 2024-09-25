use super::ContentIndexMetadata;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RawTextSearchMetadata {
    pub start_index: usize,
    pub end_index: usize,
}

impl TryFrom<ContentIndexMetadata> for RawTextSearchMetadata {
    type Error = anyhow::Error;

    fn try_from(metadata: ContentIndexMetadata) -> Result<Self, Self::Error> {
        match metadata {
            ContentIndexMetadata::RawText(metadata) => Ok(metadata),
            _ => anyhow::bail!("metadata is not from raw text"),
        }
    }
}

impl From<RawTextSearchMetadata> for ContentIndexMetadata {
    fn from(metadata: RawTextSearchMetadata) -> Self {
        ContentIndexMetadata::RawText(metadata)
    }
}

impl PartialEq for RawTextSearchMetadata {
    fn eq(&self, other: &Self) -> bool {
        self.start_index == other.start_index && self.end_index == other.end_index
    }
}

impl Eq for RawTextSearchMetadata {}

impl PartialOrd for RawTextSearchMetadata {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.start_index.partial_cmp(&other.start_index) {
            Some(std::cmp::Ordering::Equal) => self.end_index.partial_cmp(&other.end_index),
            other => other,
        }
    }
}

impl Ord for RawTextSearchMetadata {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}
