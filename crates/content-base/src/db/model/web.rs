use crate::db::model::{id::ID, PageModel};
use serde::Serialize;

#[derive(Serialize, Debug, Clone)]
pub struct WebPageModel {
    pub id: Option<ID>,
}

const WEB_PAGE_CREATE_STATEMENT: &'static str = r#"
(CREATE ONLY web CONTENT {{}}).id
"#;

impl WebPageModel {
    pub fn new(page: Vec<PageModel>) -> Self {
        Self { id: None, page }
    }

    pub async fn create_only<T>(
        client: &surrealdb::Surreal<T>,
        web_page_model: &Self,
    ) -> anyhow::Result<surrealdb::sql::Thing>
    where
        T: surrealdb::Connection,
    {
        let page_records = PageModel::create_batch(client, &web_page_model.page).await?;
        let mut resp = client.query(WEB_PAGE_CREATE_STATEMENT).await?;
        if let Err(errors_map) = crate::check_db_error_from_resp!(resp) {
            anyhow::bail!("Failed to insert web page, errors: {:?}", errors_map);
        };
        let Some(web_page_record) = resp.take::<Option<surrealdb::sql::Thing>>(0)? else {
            anyhow::bail!("Failed to insert web page, no id returned");
        };
        client
            .query("RELATE $relation_in -> contains -> $relation_outs;")
            .bind(("relation_in", web_page_record.clone()))
            .bind(("relation_outs", page_records.clone()))
            .await?;
        Ok(web_page_record)
    }

    pub fn table() -> &'static str {
        "web"
    }
}
