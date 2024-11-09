use crate::db::model::id::ID;
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct PayloadModel {
    pub id: Option<ID>,
    pub url: Option<String>,
    pub file_identifier: Option<String>,
}

const CREATE_STATEMENT: &'static str = r#"
(CREATE ONLY payload CONTENT {
    file_identifier: $file_identifier,
    url: $url
}).id
"#;

impl PayloadModel {
    pub fn url(&self) -> String {
        self.url.clone().unwrap_or_default()
    }
    pub fn file_identifier(&self) -> String {
        self.file_identifier.clone().unwrap_or_default()
    }
    pub fn create_statement() -> &'static str {
        CREATE_STATEMENT
    }
}

impl From<String> for PayloadModel {
    fn from(value: String) -> Self {
        Self {
            id: None,
            url: None,
            file_identifier: Some(value),
        }
    }
}
