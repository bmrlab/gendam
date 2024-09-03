use serde::{Deserialize, Serialize};
use crate::query::payload::SearchPayload;

#[derive(Debug, Deserialize, Serialize)]
pub struct PayloadModel {
    url: Option<String>,
    file_identifier: Option<String>,
}

impl From<SearchPayload> for PayloadModel {
    fn from(value: SearchPayload) -> Self {
        Self {
            url: None,
            file_identifier: Some(value.file_identifier),
        }
    }
}