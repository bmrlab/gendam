use super::id::ID;
use super::ModelCreate;
use async_trait::async_trait;
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

#[async_trait]
impl<T> ModelCreate<T, Self> for TextModel
where
    T: surrealdb::Connection,
{
    async fn create_only(
        client: &surrealdb::Surreal<T>,
        text: &Self,
    ) -> anyhow::Result<surrealdb::sql::Thing> {
        let mut resp = client.query(CREATE_STATEMENT).bind(text.clone()).await?;
        if let Err(errors_map) = crate::check_db_error_from_resp!(resp) {
            anyhow::bail!("Failed to insert text, errors: {:?}", errors_map);
        };
        let Some(thing) = resp.take::<Option<surrealdb::sql::Thing>>(0)? else {
            anyhow::bail!("Failed to insert text, no id returned");
        };
        Ok(thing)
    }
}

impl TextModel {
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
