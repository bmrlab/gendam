use serde::Deserialize;
use surrealdb::sql::Thing;

use super::model::id::ID;

pub(crate) mod full_text;
pub(crate) mod vector;

impl From<Thing> for ID {
    fn from(value: Thing) -> Self {
        ID::new(value.id.to_raw(), value.tb.as_str().into())
    }
}

#[derive(Debug, Deserialize)]
pub struct TextEntity {
    id: Thing,
    data: String,
    vector: Vec<f32>,
}

#[derive(Debug, Deserialize)]
pub struct ImageEntity {
    id: Thing,
    url: String,
    prompt: String,
    prompt_vector: Vec<f32>,
    vector: Vec<f32>,
}

#[derive(Debug, Deserialize)]
pub struct ItemEntity {
    id: Thing,
    text: Vec<TextEntity>,
    image: Vec<ImageEntity>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ContainRelationEntity {
    id: Thing,
    r#in: Thing,
    out: Thing,
}

impl ContainRelationEntity {
    pub fn in_id(&self) -> String {
        format!("{}:{}", self.r#in.tb, self.r#in.id.to_raw())
    }

    pub fn out_id(&self) -> String {
        format!("{}:{}", self.out.tb, self.out.id.to_raw())
    }
}

#[derive(Debug)]
pub enum SelectResultEntity {
    Text(TextEntity),
    Image(ImageEntity),
    Item(ItemEntity),
}

impl SelectResultEntity {
    pub fn id(&self) -> ID {
        match self {
            SelectResultEntity::Text(text) => ID::from(&text.id),
            SelectResultEntity::Image(image) => ID::from(&image.id),
            SelectResultEntity::Item(item) => ID::from(&item.id),
        }
    }
}
