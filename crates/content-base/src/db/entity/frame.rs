use super::{ImageEntity, TextEntity};
use crate::db::model::{
    audio::AudioFrameModel, image::ImageModel, text::TextModel, video::ImageFrameModel,
};
use serde::Deserialize;
use surrealdb::sql::Thing;

#[derive(Debug, Deserialize)]
pub struct AudioFrameEntity {
    id: Thing,
    data: Vec<TextEntity>,
    start_timestamp: usize,
    end_timestamp: usize,
}

impl From<AudioFrameEntity> for AudioFrameModel {
    fn from(value: AudioFrameEntity) -> Self {
        Self {
            id: Some(value.id.into()),
            data: value.data.into_iter().map(TextModel::from).collect(),
            start_timestamp: value.start_timestamp as f32,
            end_timestamp: value.end_timestamp as f32,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ImageFrameEntity {
    id: Thing,
    data: Vec<ImageEntity>,
    start_timestamp: usize,
    end_timestamp: usize,
}

impl From<ImageFrameEntity> for ImageFrameModel {
    fn from(value: ImageFrameEntity) -> Self {
        Self {
            id: Some(value.id.into()),
            data: value.data.into_iter().map(ImageModel::from).collect(),
            start_timestamp: value.start_timestamp as f32,
            end_timestamp: value.end_timestamp as f32,
        }
    }
}
