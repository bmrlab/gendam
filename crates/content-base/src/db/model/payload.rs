use super::ModelCreate;
use crate::db::model::id::ID;
use async_trait::async_trait;
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct PayloadModel {
    pub id: Option<ID>,
    pub url: Option<String>,
    pub file_identifier: Option<String>,
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

const PAYLOAD_CREATE_STATEMENT: &'static str = r#"
(CREATE ONLY payload CONTENT {
    file_identifier: $file_identifier,
    url: $url
}).id
"#;

#[async_trait]
impl<T> ModelCreate<T, Self> for PayloadModel
where
    T: surrealdb::Connection,
{
    async fn create_only(
        client: &surrealdb::Surreal<T>,
        payload: &Self,
    ) -> anyhow::Result<surrealdb::sql::Thing> {
        let mut resp = client
            .query(PAYLOAD_CREATE_STATEMENT)
            .bind(payload.clone())
            .await?;
        if let Err(errors_map) = crate::check_db_error_from_resp!(resp) {
            anyhow::bail!("Failed to insert payload, errors: {:?}", errors_map);
        };
        let Some(thing) = resp.take::<Option<surrealdb::sql::Thing>>(0)? else {
            anyhow::bail!("Failed to insert payload, no id returned");
        };
        tracing::debug!(id=%thing, "Payload created in surrealdb");
        Ok(thing)
    }
}

impl PayloadModel {
    pub fn url(&self) -> String {
        self.url.clone().unwrap_or_default()
    }
    pub fn file_identifier(&self) -> String {
        self.file_identifier.clone().unwrap_or_default()
    }

    pub async fn create_for_model<T>(
        client: &surrealdb::Surreal<T>,
        model_record: &surrealdb::sql::Thing,
        payload_model: &Self,
    ) -> anyhow::Result<()>
    where
        T: surrealdb::Connection,
    {
        let payload_record = Self::create_only(client, payload_model).await?;
        client
            .query("RELATE $model_record -> with -> $payload_record;")
            .bind(("model_record", model_record.clone()))
            .bind(("payload_record", payload_record))
            .await?;
        Ok(())
    }

    pub fn table() -> &'static str {
        "payload"
    }
}
