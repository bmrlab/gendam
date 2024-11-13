use crate::db::{
    entity::{ImageEntity, TextEntity},
    model::page::PageModel,
};
use serde::Deserialize;
use surrealdb::sql::Thing;

#[derive(Debug, Deserialize)]
pub struct PageEntity {
    id: Thing,
    text: Vec<TextEntity>,
    image: Vec<ImageEntity>,
    start_index: usize,
    end_index: usize,
}

impl From<PageEntity> for PageModel {
    fn from(value: PageEntity) -> Self {
        Self {
            id: Some(value.id.into()),
            start_index: value.start_index,
            end_index: value.end_index,
        }
    }
}
