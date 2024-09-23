use super::{ImageEntity, TextEntity};
use crate::db::model::{ImageModel, PageModel, TextModel};
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
            text: value.text.into_iter().map(TextModel::from).collect(),
            image: value.image.into_iter().map(ImageModel::from).collect(),
            start_index: value.start_index as i32,
            end_index: value.end_index as i32,
        }
    }
}
