use super::ModelCreate;
use super::{id::ID, ImageModel, TextModel};
use async_trait::async_trait;
use serde::Serialize;

#[derive(Serialize, Debug, Clone)]
pub struct PageModel {
    pub id: Option<ID>,
    pub start_index: usize,
    pub end_index: usize,
}

const PAGE_CREATE_STATEMENT: &'static str = r#"
(CREATE ONLY page CONTENT {{
    start_index: $start_index,
    end_index: $end_index
}}).id
"#;

#[async_trait]
impl<T> ModelCreate<T, (Self, Vec<TextModel>, Vec<ImageModel>)> for PageModel
where
    T: surrealdb::Connection,
{
    async fn create_only(
        client: &surrealdb::Surreal<T>,
        (page, page_texts, page_images): &(Self, Vec<TextModel>, Vec<ImageModel>),
    ) -> anyhow::Result<surrealdb::sql::Thing> {
        let text_records = TextModel::create_batch(client, page_texts).await?;
        let image_records = ImageModel::create_batch(client, page_images).await?;
        let mut resp = client
            .query(PAGE_CREATE_STATEMENT)
            .bind(page.clone())
            .await?;
        if let Err(errors_map) = crate::check_db_error_from_resp!(resp) {
            anyhow::bail!("Failed to insert video, errors: {:?}", errors_map);
        };
        let Some(page_record) = resp.take::<Option<surrealdb::sql::Thing>>(0)? else {
            anyhow::bail!("Failed to insert video, no id returned");
        };
        tracing::debug!(id=%page_record, "Page created in surrealdb");
        let all_records = Vec::new()
            .into_iter()
            .chain(text_records.into_iter())
            .chain(image_records.into_iter())
            .collect::<Vec<_>>();
        client
            .query("RELATE $relation_in -> contains -> $relation_outs;")
            .bind(("relation_in", page_record.clone()))
            .bind(("relation_outs", all_records))
            .await?;
        Ok(page_record)
    }
}

impl PageModel {
    pub fn table() -> &'static str {
        "page"
    }
}
