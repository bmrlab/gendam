use super::SearchMetadata;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AudioSearchMetadata {
    pub start_timestamp: i64,
    pub end_timestamp: i64,
}

impl AudioSearchMetadata {
    pub fn new(start_timestamp: i64, end_timestamp: i64) -> Self {
        Self {
            start_timestamp,
            end_timestamp,
        }
    }
}

impl TryFrom<SearchMetadata> for AudioSearchMetadata {
    type Error = anyhow::Error;

    fn try_from(metadata: SearchMetadata) -> Result<Self, Self::Error> {
        match metadata {
            SearchMetadata::Audio(metadata) => Ok(metadata),
            _ => anyhow::bail!("metadata is not from audio"),
        }
    }
}

impl From<AudioSearchMetadata> for SearchMetadata {
    fn from(metadata: AudioSearchMetadata) -> Self {
        SearchMetadata::Audio(metadata)
    }
}

impl PartialEq for AudioSearchMetadata {
    fn eq(&self, other: &Self) -> bool {
        self.start_timestamp == other.start_timestamp && self.end_timestamp == other.end_timestamp
    }
}

impl Eq for AudioSearchMetadata {}

impl PartialOrd for AudioSearchMetadata {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.start_timestamp.partial_cmp(&other.start_timestamp) {
            Some(std::cmp::Ordering::Equal) => self.end_timestamp.partial_cmp(&other.end_timestamp),
            other => other,
        }
    }
}

impl Ord for AudioSearchMetadata {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}
