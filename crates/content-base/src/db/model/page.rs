use super::{id::ID, ImageModel, TextModel};
use serde::Serialize;

#[derive(Serialize, Debug, Clone)]
pub struct PageModel {
    pub id: Option<ID>,
    pub text: Vec<TextModel>,
    pub image: Vec<ImageModel>,
    pub start_index: i32,
    pub end_index: i32,
}

const PAGE_CREATE_STATEMENT: &'static str = r#"
(CREATE ONLY page CONTENT {{
    text: $texts,
    image: $images,
    start_index: $start_index,
    end_index: $end_index
}}).id
"#;

impl PageModel {
    pub async fn create_only<T>(
        client: &surrealdb::Surreal<T>,
        page_model: &Self,
    ) -> anyhow::Result<surrealdb::sql::Thing>
    where
        T: surrealdb::Connection,
    {
        let text_records = TextModel::create_batch(client, &page_model.text).await?;
        let image_records = ImageModel::create_batch(client, &page_model.image).await?;
        let mut resp = client
            .query(PAGE_CREATE_STATEMENT)
            .bind(("texts", text_records.clone()))
            .bind(("images", image_records.clone()))
            .bind(page_model.clone())
            .await?;
        if let Err(errors_map) = crate::check_db_error_from_resp!(resp) {
            anyhow::bail!("Failed to insert video, errors: {:?}", errors_map);
        };
        let Some(page_record) = resp.take::<Option<surrealdb::sql::Thing>>(0)? else {
            anyhow::bail!("Failed to insert video, no id returned");
        };
        let all_frames = Vec::new()
            .into_iter()
            .chain(text_records.into_iter())
            .chain(image_records.into_iter())
            .collect::<Vec<_>>();
        client
            .query("RELATE $relation_in -> contains -> $relation_outs;")
            .bind(("relation_in", page_record.clone()))
            .bind(("relation_outs", all_frames))
            .await?;
        Ok(page_record)
    }

    pub async fn create_batch<T>(
        client: &surrealdb::Surreal<T>,
        pages_models: &Vec<Self>,
    ) -> anyhow::Result<Vec<surrealdb::sql::Thing>>
    where
        T: surrealdb::Connection,
    {
        let futures = pages_models
            .into_iter()
            .map(|page_model| Self::create_only(client, page_model))
            .collect::<Vec<_>>();
        let results = crate::collect_async_results!(futures);
        results
    }

    pub fn table() -> &'static str {
        "page"
    }
}
