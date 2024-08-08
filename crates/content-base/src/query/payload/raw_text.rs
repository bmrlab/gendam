use super::SearchMetadata;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RawTextSearchMetadata {
    pub index: usize,
}

impl TryFrom<SearchMetadata> for RawTextSearchMetadata {
    type Error = anyhow::Error;

    fn try_from(metadata: SearchMetadata) -> Result<Self, Self::Error> {
        match metadata {
            SearchMetadata::RawText(metadata) => Ok(metadata),
            _ => anyhow::bail!("metadata is not from raw text"),
        }
    }
}

impl From<RawTextSearchMetadata> for SearchMetadata {
    fn from(metadata: RawTextSearchMetadata) -> Self {
        SearchMetadata::RawText(metadata)
    }
}

impl PartialEq for RawTextSearchMetadata {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}

impl Eq for RawTextSearchMetadata {}

impl PartialOrd for RawTextSearchMetadata {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.index.cmp(&other.index))
    }
}

impl Ord for RawTextSearchMetadata {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}
