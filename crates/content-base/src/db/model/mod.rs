use serde::Serialize;
use crate::db::model::audio::AudioModel;
use crate::db::model::document::DocumentModel;
use crate::db::model::id::ID;
use crate::db::model::video::VideoModel;
use crate::db::model::web::WebPageModel;

pub mod audio;
pub mod document;
pub mod id;
pub mod payload;
pub mod video;
pub mod web;

#[derive(Debug, Serialize)]
pub struct ImageModel {
    pub id: Option<ID>,
    pub prompt: String,
    pub vector: Vec<f32>,
    pub prompt_vector: Vec<f32>,
}

#[derive(Debug, Serialize)]
pub struct TextModel {
    pub id: Option<ID>,
    pub data: String,
    pub vector: Vec<f32>,
    pub en_data: String,
    pub en_vector: Vec<f32>,
}

#[derive(Debug)]
pub struct PageModel {
    pub id: Option<ID>,
    pub text: Vec<TextModel>,
    pub image: Vec<ImageModel>,
    pub start_index: i32,
    pub end_index: i32,
}

#[derive(Debug)]
pub struct ItemModel {
    pub id: Option<ID>,
    text: Vec<TextModel>,
    image: Vec<ImageModel>,
}

#[derive(Debug)]
pub struct PayloadModel {
    pub id: Option<ID>,
    pub file_identifier: Option<String>,
    pub url: Option<String>,
}

impl PayloadModel {
    pub fn url(&self) -> String {
        self.url.clone().unwrap_or_default()
    }
    pub fn file_identifier(&self) -> String {
        self.file_identifier.clone().unwrap_or_default()
    }
}

#[derive(Debug)]
pub enum SelectResultModel {
    Text(TextModel),
    Image(ImageModel),
    Item(ItemModel),
    Audio(AudioModel),
    Video(VideoModel),
    WebPage(WebPageModel),
    Document(DocumentModel),
    Payload(PayloadModel),
}
