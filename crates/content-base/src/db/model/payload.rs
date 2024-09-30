use crate::query::payload::ContentIndexPayload;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct PayloadModel {
    url: Option<String>,
    file_identifier: Option<String>,
}

impl From<ContentIndexPayload> for PayloadModel {
    fn from(value: ContentIndexPayload) -> Self {
        Self {
            url: None,
            file_identifier: Some(value.file_identifier),
        }
    }
}
