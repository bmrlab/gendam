use super::{
    id::ID, image::ImageModel, page::PageModel, text::TextModel, ModelCreate, ModelDelete,
};
use async_trait::async_trait;
use serde::Serialize;

#[derive(Serialize, Debug, Clone)]
pub struct WebPageModel {
    pub id: Option<ID>,
}

const WEB_PAGE_CREATE_STATEMENT: &'static str = r#"
(CREATE ONLY web_page CONTENT {{}}).id
"#;

#[async_trait]
impl<T> ModelCreate<T, (Self, Vec<(PageModel, Vec<TextModel>, Vec<ImageModel>)>)> for WebPageModel
where
    T: surrealdb::Connection,
{
    async fn create_only(
        client: &surrealdb::Surreal<T>,
        (_web_page, pages): &(Self, Vec<(PageModel, Vec<TextModel>, Vec<ImageModel>)>),
    ) -> anyhow::Result<surrealdb::sql::Thing> {
        let page_records = PageModel::create_batch(client, pages).await?;
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
}

const WEB_PAGE_DELETE_STATEMENT: &'static str = r#"
LET $v = (
    SELECT
        ->contains->page AS pages,
        ->contains->page->contains->text AS texts,
        ->contains->page->contains->image AS images,
        ->with->payload AS payload,
        id
    FROM ONLY $record
);
let $ids = array::flatten([$v.images, $v.texts, $v.pages, $v.payload, $v.id]);
DELETE $ids;
"#;

#[async_trait]
impl<T> ModelDelete<T> for WebPageModel
where
    T: surrealdb::Connection,
{
    async fn delete_cascade(
        client: &surrealdb::Surreal<T>,
        record: &surrealdb::sql::Thing,
    ) -> anyhow::Result<()> {
        let mut resp = client
            .query(WEB_PAGE_DELETE_STATEMENT)
            .bind(("record", record.clone()))
            .await?;
        if let Err(errors_map) = crate::check_db_error_from_resp!(resp) {
            anyhow::bail!("Failed to delete web page, errors: {:?}", errors_map);
        };
        Ok(())
    }
}

impl WebPageModel {
    pub fn table() -> &'static str {
        "web_page"
    }
}
