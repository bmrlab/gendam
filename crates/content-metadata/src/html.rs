use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HTMLMetadata {
    pub source_url: String,
}
