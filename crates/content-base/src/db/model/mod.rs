use crate::db::model::audio::AudioModel;
use crate::db::model::document::DocumentModel;
use crate::db::model::id::ID;
use crate::db::model::video::VideoModel;
use crate::db::model::web::WebPageModel;
use educe::Educe;
use serde::Serialize;

pub mod audio;
pub mod document;
pub mod id;
pub mod payload;
pub mod video;
pub mod web;

#[derive(Serialize, Educe, Clone)]
#[educe(Debug)]
pub struct ImageModel {
    pub id: Option<ID>,
    pub prompt: String,

    #[educe(Debug(ignore))]
    pub vector: Vec<f32>,

    #[educe(Debug(ignore))]
    pub prompt_vector: Vec<f32>,
}

#[derive(Serialize, Educe, Clone)]
#[educe(Debug)]
pub struct TextModel {
    pub id: Option<ID>,
    pub data: String,

    #[educe(Debug(ignore))]
    pub vector: Vec<f32>,
    #[educe(Debug(ignore))]
    pub en_data: String,

    #[educe(Debug(ignore))]
    pub en_vector: Vec<f32>,
}

#[derive(Debug, Clone)]
pub struct PageModel {
    pub id: Option<ID>,
    pub text: Vec<TextModel>,
    pub image: Vec<ImageModel>,
    pub start_index: i32,
    pub end_index: i32,
}

#[derive(Debug, Clone)]
pub struct ItemModel {
    pub id: Option<ID>,
    text: Vec<TextModel>,
    image: Vec<ImageModel>,
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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

impl SelectResultModel {
    pub fn id(&self) -> Option<ID> {
        match self {
            SelectResultModel::Text(data) => data.id.clone(),
            SelectResultModel::Image(data) => data.id.clone(),
            SelectResultModel::Item(data) => data.id.clone(),
            SelectResultModel::Audio(data) => data.id.clone(),
            SelectResultModel::Video(data) => data.id.clone(),
            SelectResultModel::WebPage(data) => data.id.clone(),
            SelectResultModel::Document(data) => data.id.clone(),
            SelectResultModel::Payload(data) => data.id.clone(),
        }
    }

    pub fn hit_text(&self) -> Option<String> {
        match self {
            SelectResultModel::Text(text) => Some(text.data.clone()),
            SelectResultModel::Image(image) => Some(image.prompt.clone()),
            SelectResultModel::Audio(audio) => {
                let mut text = vec![];
                for frame in &audio.audio_frame {
                    for data in &frame.data {
                        text.push(data.data.clone());
                    }
                }
                Some(text.join("\n"))
            }
            SelectResultModel::Video(video) => {
                let mut text = vec![];
                for frame in &video.audio_frame {
                    for data in &frame.data {
                        text.push(data.data.clone());
                    }
                }
                for frame in &video.image_frame {
                    for data in &frame.data {
                        text.push(data.prompt.clone());
                    }
                }
                Some(text.join("\n"))
            }
            SelectResultModel::WebPage(web) => {
                let mut text = vec![];
                for data in &web.page {
                    text.push(
                        data.text
                            .iter()
                            .map(|x| x.data.clone())
                            .collect::<Vec<String>>()
                            .join("\n"),
                    );
                }
                Some(text.join("\n"))
            }
            SelectResultModel::Document(document) => {
                let mut text = vec![];
                for data in &document.page {
                    text.push(
                        data.text
                            .iter()
                            .map(|x| x.data.clone())
                            .collect::<Vec<String>>()
                            .join("\n"),
                    );
                }
                Some(text.join("\n"))
            }
            _ => None,
        }
    }
}
