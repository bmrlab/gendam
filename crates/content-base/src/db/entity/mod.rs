use crate::db::model::{
    audio::{AudioFrameModel, AudioModel},
    document::DocumentModel,
    id::ID,
    image::ImageModel,
    payload::PayloadModel,
    text::TextModel,
    video::{ImageFrameModel, VideoModel},
    web::WebPageModel,
    PageModel, SelectResultModel,
};
use frame::{AudioFrameEntity, ImageFrameEntity};
use page::PageEntity;
use serde::Deserialize;
use surrealdb::sql::Thing;

pub(crate) mod frame;
pub(crate) mod full_text;
pub(crate) mod page;
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

impl From<TextEntity> for TextModel {
    fn from(value: TextEntity) -> Self {
        Self {
            id: Some(ID::from(&value.id)),
            data: value.data,
            vector: value.vector,
            en_data: value.en_data,
            en_vector: value.en_vector,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ImageEntity {
    id: Thing,
    vector: Vec<f32>,
    prompt: String,
    prompt_vector: Vec<f32>,
}

impl From<ImageEntity> for ImageModel {
    fn from(value: ImageEntity) -> Self {
        Self {
            id: Some(ID::from(&value.id)),
            prompt: value.prompt,
            vector: value.vector,
            prompt_vector: value.prompt_vector,
        }
    }
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

impl From<VideoEntity> for VideoModel {
    fn from(value: VideoEntity) -> Self {
        Self {
            id: Some(ID::from(&value.id)),
            image_frame: value
                .image_frame
                .into_iter()
                .map(ImageFrameModel::from)
                .collect(),
            audio_frame: value
                .audio_frame
                .into_iter()
                .map(AudioFrameModel::from)
                .collect(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct WebPageEntity {
    id: Thing,
    page: Vec<PageEntity>,
}

impl From<WebPageEntity> for WebPageModel {
    fn from(value: WebPageEntity) -> Self {
        Self {
            id: Some(ID::from(&value.id)),
            page: value.page.into_iter().map(PageModel::from).collect(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct DocumentEntity {
    id: Thing,
    page: Vec<PageEntity>,
}

impl From<DocumentEntity> for DocumentModel {
    fn from(value: DocumentEntity) -> Self {
        Self {
            id: Some(ID::from(&value.id)),
            page: value.page.into_iter().map(PageModel::from).collect(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct PayloadEntity {
    id: Thing,
    file_identifier: Option<String>,
    url: Option<String>,
}

impl From<PayloadEntity> for PayloadModel {
    fn from(value: PayloadEntity) -> Self {
        Self {
            id: Some(ID::from(&value.id)),
            file_identifier: value.file_identifier,
            url: value.url,
        }
    }
}

impl PayloadEntity {
    pub fn url(&self) -> String {
        self.url.clone().unwrap_or_default()
    }
    pub fn file_identifier(&self) -> String {
        self.file_identifier.clone().unwrap_or_default()
    }
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
    Payload(PayloadEntity),
}

impl From<SelectResultEntity> for SelectResultModel {
    fn from(value: SelectResultEntity) -> Self {
        match value {
            SelectResultEntity::Text(text) => SelectResultModel::Text(TextModel::from(text)),
            SelectResultEntity::Image(image) => SelectResultModel::Image(ImageModel::from(image)),
            SelectResultEntity::Audio(audio) => SelectResultModel::Audio(AudioModel {
                id: Some(ID::from(&audio.id)),
                audio_frame: audio.frame.into_iter().map(AudioFrameModel::from).collect(),
            }),
            SelectResultEntity::Video(video) => SelectResultModel::Video(VideoModel::from(video)),
            SelectResultEntity::WebPage(web) => SelectResultModel::WebPage(WebPageModel::from(web)),
            SelectResultEntity::Document(document) => {
                SelectResultModel::Document(DocumentModel::from(document))
            }
            SelectResultEntity::Payload(payload) => {
                SelectResultModel::Payload(PayloadModel::from(payload))
            }
            _ => unimplemented!(),
        }
    }
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
            SelectResultEntity::Payload(payload) => ID::from(&payload.id),
        }
    }
}
