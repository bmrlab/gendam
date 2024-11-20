use super::{ModelCreate, ModelDelete};
use crate::db::model::id::ID;
use async_trait::async_trait;
use educe::Educe;
use serde::Serialize;

#[derive(Serialize, Educe, Clone)]
#[educe(Debug)]
pub struct ImageModel {
    pub id: Option<ID>,

    #[educe(Debug(ignore))]
    pub embedding: Vec<f32>,

    pub caption: String,
    #[educe(Debug(ignore))]
    pub caption_embedding: Vec<f32>,
}

const CREATE_STATEMENT: &'static str = r#"
(CREATE ONLY image CONTENT {
    embedding: $embedding,
    caption: $caption,
    caption_embedding: $caption_embedding
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
        tracing::debug!(id=%thing, "Image created in surrealdb");
        Ok(thing)
    }
}

const IMAGE_DELETE_STATEMENT: &'static str = r#"
LET $v = (
    SELECT
        ->with->payload AS payload,
        id
    FROM ONLY $record
);
let $ids = array::flatten([$v.payload, $v.id]);
DELETE $ids;
"#;

#[async_trait]
impl<T> ModelDelete<T> for ImageModel
where
    T: surrealdb::Connection,
{
    async fn delete_cascade(
        client: &surrealdb::Surreal<T>,
        record: &surrealdb::sql::Thing,
    ) -> anyhow::Result<()> {
        let mut resp = client
            .query(IMAGE_DELETE_STATEMENT)
            .bind(("record", record.clone()))
            .await?;
        if let Err(errors_map) = crate::check_db_error_from_resp!(resp) {
            anyhow::bail!("Failed to delete image, errors: {:?}", errors_map);
        };
        Ok(())
    }
}

impl ImageModel {
    pub fn table() -> &'static str {
        "image"
    }

    pub fn text_embedding_columns() -> Vec<&'static str> {
        vec!["caption_embedding"]
    }

    pub fn vision_embedding_columns() -> Vec<&'static str> {
        vec!["embedding"]
    }

    pub fn full_text_columns() -> Vec<&'static str> {
        vec!["caption"]
    }
}
