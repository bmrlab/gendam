use frame::{AudioFrameEntity, ImageFrameEntity};
use page::PageEntity;
use serde::Deserialize;
use surrealdb::sql::Thing;

use super::model::id::ID;

pub(crate) mod frame;
pub(crate) mod full_text;
pub(crate) mod page;
pub(crate) mod payload;
pub(crate) mod relation;
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
    en_data: String,
    en_vector: Vec<f32>,
}

#[derive(Debug, Deserialize)]
pub struct ImageEntity {
    id: Thing,
    vector: Vec<f32>,
    prompt: String,
    prompt_vector: Vec<f32>,
}

#[derive(Debug, Deserialize)]
pub struct ItemEntity {
    id: Thing,
    text: Vec<TextEntity>,
    image: Vec<ImageEntity>,
}

#[derive(Debug, Deserialize)]
pub struct AudioEntity {
    id: Thing,
    frame: Vec<AudioFrameEntity>,
}

#[derive(Debug, Deserialize)]
pub struct VideoEntity {
    id: Thing,
    image_frame: Vec<ImageFrameEntity>,
    audio_frame: Vec<AudioFrameEntity>,
}

#[derive(Debug, Deserialize)]
pub struct WebPageEntity {
    id: Thing,
    page: Vec<PageEntity>,
}

#[derive(Debug, Deserialize)]
pub struct DocumentEntity {
    id: Thing,
    page: Vec<PageEntity>,
}

#[derive(Debug)]
pub enum SelectResultEntity {
    Text(TextEntity),
    Image(ImageEntity),
    Item(ItemEntity),
    Audio(AudioEntity),
    Video(VideoEntity),
    WebPage(WebPageEntity),
    Document(DocumentEntity),
}

impl SelectResultEntity {
    pub fn id(&self) -> ID {
        match self {
            SelectResultEntity::Text(text) => ID::from(&text.id),
            SelectResultEntity::Image(image) => ID::from(&image.id),
            SelectResultEntity::Item(item) => ID::from(&item.id),
            SelectResultEntity::Audio(audio) => ID::from(&audio.id),
            SelectResultEntity::Video(video) => ID::from(&video.id),
            SelectResultEntity::WebPage(web) => ID::from(&web.id),
            SelectResultEntity::Document(document) => ID::from(&document.id),
        }
    }
}
