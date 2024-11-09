use crate::db::model::id::ID;
use educe::Educe;
use serde::Serialize;

#[derive(Serialize, Educe, Clone)]
#[educe(Debug)]
pub struct ImageModel {
    pub id: Option<ID>,
    pub prompt: String,
    #[educe(Debug(ignore))]
    pub vector: Vec<f32>,
    #[educe(Debug(ignore))]
    pub prompt_vector: Vec<f32>,
}

const CREATE_STATEMENT: &'static str = r#"
(CREATE ONLY image CONTENT {
    prompt: $prompt,
    vector: $vector,
    prompt_vector: $prompt_vector
}).id
"#;

impl ImageModel {
    // pub fn create_statement() -> &'static str {
    //     CREATE_STATEMENT
    // }

    pub async fn create_only<T>(
        client: &surrealdb::Surreal<T>,
        image_model: &Self,
    ) -> anyhow::Result<surrealdb::sql::Thing>
    where
        T: surrealdb::Connection,
    {
        let mut resp = client
            .query(CREATE_STATEMENT)
            .bind(image_model.clone())
            .await?;
        if let Err(errors_map) = crate::check_db_error_from_resp!(resp) {
            anyhow::bail!("Failed to insert image, errors: {:?}", errors_map);
        };
        let Some(thing) = resp.take::<Option<surrealdb::sql::Thing>>(0)? else {
            anyhow::bail!("Failed to insert image, no id returned");
        };
        Ok(thing)
    }

    pub async fn create_batch<T>(
        client: &surrealdb::Surreal<T>,
        image_models: &Vec<Self>,
    ) -> anyhow::Result<Vec<surrealdb::sql::Thing>>
    where
        T: surrealdb::Connection,
    {
        let futures = image_models
            .into_iter()
            .map(|image_model| Self::create_only(client, image_model))
            .collect::<Vec<_>>();
        let results = crate::collect_async_results!(futures);
        results
    }

    pub fn table() -> &'static str {
        "image"
    }

    pub fn text_vector_columns() -> Vec<&'static str> {
        vec!["prompt_vector"]
    }

    pub fn vision_vector_columns() -> Vec<&'static str> {
        vec!["vector"]
    }

    pub fn full_text_columns() -> Vec<&'static str> {
        vec!["prompt"]
    }
}
