use super::ModelCreate;
use crate::db::model::id::ID;
use async_trait::async_trait;
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

#[async_trait]
impl<T> ModelCreate<T, Self> for ImageModel
where
    T: surrealdb::Connection,
{
    async fn create_only(
        client: &surrealdb::Surreal<T>,
        image: &Self,
    ) -> anyhow::Result<surrealdb::sql::Thing> {
        let mut resp = client.query(CREATE_STATEMENT).bind(image.clone()).await?;
        if let Err(errors_map) = crate::check_db_error_from_resp!(resp) {
            anyhow::bail!("Failed to insert image, errors: {:?}", errors_map);
        };
        let Some(thing) = resp.take::<Option<surrealdb::sql::Thing>>(0)? else {
            anyhow::bail!("Failed to insert image, no id returned");
        };
        Ok(thing)
    }
}

impl ImageModel {
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
