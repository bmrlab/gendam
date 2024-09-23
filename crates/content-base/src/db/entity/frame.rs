use super::{ImageEntity, TextEntity};
use crate::db::model::audio::AudioFrameModel;
use crate::db::model::video::ImageFrameModel;
use crate::db::model::{ImageModel, TextModel};
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
            data: value.data.into_iter().map(ImageModel::from).collect(),
            start_timestamp: value.start_timestamp as f32,
            end_timestamp: value.end_timestamp as f32,
        }
    }
}
