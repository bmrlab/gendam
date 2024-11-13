use crate::check_db_error_from_resp;
use crate::db::{
    model::{
        audio::AudioModel, document::DocumentModel, image::ImageModel, video::VideoModel,
        web::WebPageModel, ModelDelete,
    },
    DB,
};

const CONTENT_TYPE_LOOKUP_QUERY: &'static str = r#"
(
    SELECT
    <-with[0].in.id as id
    FROM ONLY payload
    WHERE file_identifier = $file_identifier LIMIT 1
).id;
"#;

impl DB {
    pub async fn delete_by_file_identifier(&self, file_identifier: &str) -> anyhow::Result<()> {
        let mut resp = self
            .client
            .query(CONTENT_TYPE_LOOKUP_QUERY)
            .bind(("file_identifier", file_identifier.to_string()))
            .await?;
        check_db_error_from_resp!(resp)
            .map_err(|errors_map| anyhow::anyhow!("content_type lookup error: {:?}", errors_map))?;
        let record = match resp.take::<Option<surrealdb::sql::Thing>>(0)? {
            Some(record) => record,
            _ => {
                tracing::warn!("No record found for file_identifier: {}", file_identifier);
                return Ok(());
            }
        };
        match record.tb.as_str() {
            "image" => {
                ImageModel::delete_cascade(&self.client, &record).await?;
            }
            "audio" => {
                AudioModel::delete_cascade(&self.client, &record).await?;
            }
            "video" => {
                VideoModel::delete_cascade(&self.client, &record).await?;
            }
            "document" => {
                DocumentModel::delete_cascade(&self.client, &record).await?;
            }
            "web" => {
                WebPageModel::delete_cascade(&self.client, &record).await?;
            }
            _ => {
                tracing::warn!("unexpected content type: {}", record.tb.as_str());
                return Ok(());
            }
        };

        Ok(())
    }
}
