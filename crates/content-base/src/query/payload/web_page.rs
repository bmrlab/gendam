use super::SearchMetadata;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WebPageSearchMetadata {
    pub index: usize,
}

impl TryFrom<SearchMetadata> for WebPageSearchMetadata {
    type Error = anyhow::Error;

    fn try_from(metadata: SearchMetadata) -> Result<Self, Self::Error> {
        match metadata {
            SearchMetadata::WebPage(metadata) => Ok(metadata),
            _ => anyhow::bail!("metadata is not from raw text"),
        }
    }
}

impl From<WebPageSearchMetadata> for SearchMetadata {
    fn from(metadata: WebPageSearchMetadata) -> Self {
        SearchMetadata::WebPage(metadata)
    }
}

impl PartialEq for WebPageSearchMetadata {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}

impl Eq for WebPageSearchMetadata {}

impl PartialOrd for WebPageSearchMetadata {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.index.cmp(&other.index))
    }
}

impl Ord for WebPageSearchMetadata {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}
