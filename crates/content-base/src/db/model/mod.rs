pub mod audio;
pub mod document;
pub mod id;
pub mod image;
pub mod page;
pub mod payload;
pub mod text;
pub mod video;
pub mod web;
use self::{
    audio::{AudioFrameModel, AudioModel},
    document::DocumentModel,
    id::ID,
    image::ImageModel,
    payload::PayloadModel,
    text::TextModel,
    video::VideoModel,
    web::WebPageModel,
};
use async_trait::async_trait;
use serde::Serialize;

#[derive(Serialize, Debug, Clone)]
pub enum SelectResultModel {
    Text(TextModel),
    Image(ImageModel),
    Audio(AudioModel),
    Video(VideoModel),
    WebPage(WebPageModel),
    Document(DocumentModel),
    Payload(PayloadModel),
}

impl SelectResultModel {
    pub fn id(&self) -> Option<ID> {
        match self {
            SelectResultModel::Text(data) => data.id.clone(),
            SelectResultModel::Image(data) => data.id.clone(),
            SelectResultModel::Audio(data) => data.id.clone(),
            SelectResultModel::Video(data) => data.id.clone(),
            SelectResultModel::WebPage(data) => data.id.clone(),
            SelectResultModel::Document(data) => data.id.clone(),
            SelectResultModel::Payload(data) => data.id.clone(),
        }
    }

    fn is_within_range<T>(start: T, end: T, range: (T, T)) -> bool
    where
        T: PartialOrd + Copy,
    {
        start >= range.0 && end <= range.1
    }
}

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
