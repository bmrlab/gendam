use super::id::ID;
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
    // pub fn create_statement() -> &'static str {
    //     CREATE_STATEMENT
    // }

    pub async fn create_only<T>(
        client: &surrealdb::Surreal<T>,
        text_model: &Self,
    ) -> anyhow::Result<surrealdb::sql::Thing>
    where
        T: surrealdb::Connection,
    {
        let mut resp = client
            .query(CREATE_STATEMENT)
            .bind(text_model.clone())
            .await?;
        if let Err(errors_map) = crate::check_db_error_from_resp!(resp) {
            anyhow::bail!("Failed to insert text, errors: {:?}", errors_map);
        };
        let Some(thing) = resp.take::<Option<surrealdb::sql::Thing>>(0)? else {
            anyhow::bail!("Failed to insert text, no id returned");
        };
        Ok(thing)
    }

    pub async fn create_batch<T>(
        client: &surrealdb::Surreal<T>,
        text_models: &Vec<Self>,
    ) -> anyhow::Result<Vec<surrealdb::sql::Thing>>
    where
        T: surrealdb::Connection,
    {
        let futures = text_models
            .into_iter()
            .map(|text_model| Self::create_only(client, text_model))
            .collect::<Vec<_>>();
        let results = crate::collect_async_results!(futures);
        results
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
