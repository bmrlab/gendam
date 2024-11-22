pub mod audio;
pub mod document;
pub mod id;
pub mod image;
pub mod page;
pub mod payload;
pub mod text;
pub mod video;
pub mod web_page;
use async_trait::async_trait;
use serde::Serialize;

#[async_trait]
pub trait ModelCreate<T, TItem>
where
    T: surrealdb::Connection,
    TItem: Serialize + Clone + Send + Sync + 'static,
{
    async fn create_only(
        client: &surrealdb::Surreal<T>,
        item: &TItem,
    ) -> anyhow::Result<surrealdb::sql::Thing>;

    async fn create_batch(
        client: &surrealdb::Surreal<T>,
        items: &Vec<TItem>,
    ) -> anyhow::Result<Vec<surrealdb::sql::Thing>> {
        let futures = items
            .into_iter()
            .map(|item| Self::create_only(client, item))
            .collect::<Vec<_>>();
        let results = crate::collect_async_results!(futures);
        results
    }
}

#[async_trait]
pub trait ModelDelete<T>
where
    T: surrealdb::Connection,
{
    async fn delete_cascade(
        client: &surrealdb::Surreal<T>,
        record: &surrealdb::sql::Thing,
    ) -> anyhow::Result<()>;
}
