use crate::db::model::{id::ID, PageModel};
use serde::Serialize;

#[derive(Serialize, Debug, Clone)]
pub struct DocumentModel {
    pub id: Option<ID>,
    pub page: Vec<PageModel>,
}

const DOCUMENT_CREATE_STATEMENT: &'static str = r#"
(CREATE ONLY document CONTENT {{
    page: $pages,
}}).id
"#;

impl DocumentModel {
    pub fn new(page: Vec<PageModel>) -> Self {
        Self { id: None, page }
    }

    pub async fn create_only<T>(
        client: &surrealdb::Surreal<T>,
        document_model: &Self,
    ) -> anyhow::Result<surrealdb::sql::Thing>
    where
        T: surrealdb::Connection,
    {
        let page_records = PageModel::create_batch(client, &document_model.page).await?;
        let mut resp = client
            .query(DOCUMENT_CREATE_STATEMENT)
            .bind(("pages", page_records.clone()))
            .await?;
        if let Err(errors_map) = crate::check_db_error_from_resp!(resp) {
            anyhow::bail!("Failed to insert document, errors: {:?}", errors_map);
        };
        let Some(document_record) = resp.take::<Option<surrealdb::sql::Thing>>(0)? else {
            anyhow::bail!("Failed to insert document, no id returned");
        };
        client
            .query("RELATE $relation_in -> contains -> $relation_outs;")
            .bind(("relation_in", document_record.clone()))
            .bind(("relation_outs", page_records.clone()))
            .await?;
        Ok(document_record)
    }

    pub fn table() -> &'static str {
        "document"
    }
}
