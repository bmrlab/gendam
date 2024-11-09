use crate::db::model::id::ID;
use educe::Educe;
use serde::Serialize;

#[derive(Serialize, Educe, Clone)]
#[educe(Debug)]
pub struct TextModel {
    pub id: Option<ID>,
    pub data: String,
    #[educe(Debug(ignore))]
    pub vector: Vec<f32>,
    #[educe(Debug(ignore))]
    pub en_data: String,
    #[educe(Debug(ignore))]
    pub en_vector: Vec<f32>,
}

const CREATE_STATEMENT: &'static str = r#"
(CREATE ONLY text CONTENT {
    data: $data,
    vector: $vector,
    en_data: $en_data,
    en_vector: $en_vector
}).id
"#;

impl TextModel {
    pub fn create_statement() -> &'static str {
        CREATE_STATEMENT
    }

    pub fn table() -> &'static str {
        "text"
    }

    pub fn text_vector_columns() -> Vec<&'static str> {
        vec!["vector", "en_vector"]
    }

    pub fn full_text_columns() -> Vec<&'static str> {
        vec!["data", "en_data"]
    }
}
