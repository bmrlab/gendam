use crate::db::model::audio::{AudioFrameModel, AudioModel};
use crate::db::model::document::DocumentModel;
use crate::db::model::id::ID;
use crate::db::model::payload::PayloadModel;
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

    fn is_within_range<T>(start: T, end: T, range: (T, T)) -> bool
    where
        T: PartialOrd + Copy,
    {
        start >= range.0 && end <= range.1
    }

    fn collect_hit_text_from_audio_frame(
        frame: &Vec<AudioFrameModel>,
        range: (usize, usize),
    ) -> Vec<String> {
        let mut text = vec![];
        for frame in frame {
            if Self::is_within_range(
                frame.start_timestamp as usize,
                frame.end_timestamp as usize,
                range,
            ) {
                for data in &frame.data {
                    text.push(data.data.clone());
                }
            }
        }
        text
    }

    fn collect_hit_text_from_page(page: &Vec<PageModel>, range: (usize, usize)) -> Vec<String> {
        let mut text = vec![];
        for data in page {
            if Self::is_within_range(data.start_index as usize, data.end_index as usize, range) {
                text.extend(
                    data.text
                        .iter()
                        .map(|x| x.data.clone())
                        .collect::<Vec<String>>(),
                );
            }
        }
        text
    }

    pub fn hit_text(&self, range: Option<(usize, usize)>) -> Option<String> {
        let range = range.unwrap_or((0, u128::MAX as usize));
        match self {
            SelectResultModel::Text(text) => Some(text.data.clone()),
            SelectResultModel::Image(image) => Some(image.prompt.clone()),
            SelectResultModel::Audio(audio) => {
                Some(Self::collect_hit_text_from_audio_frame(&audio.audio_frame, range).join("\n"))
            }
            SelectResultModel::Video(video) => {
                let mut text = vec![];
                text.extend(Self::collect_hit_text_from_audio_frame(
                    &video.audio_frame,
                    range,
                ));
                for frame in &video.image_frame {
                    if Self::is_within_range(
                        frame.start_timestamp as usize,
                        frame.end_timestamp as usize,
                        range,
                    ) {
                        for data in &frame.data {
                            text.push(data.prompt.clone());
                        }
                    }
                }
                Some(text.join("\n"))
            }
            SelectResultModel::WebPage(web) => {
                Some(Self::collect_hit_text_from_page(&web.page, range).join("\n"))
            }
            SelectResultModel::Document(document) => {
                Some(Self::collect_hit_text_from_page(&document.page, range).join("\n"))
            }
            _ => None,
        }
    }
}
