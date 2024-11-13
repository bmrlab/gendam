use super::{ImageEntity, TextEntity};
use crate::db::model::{audio::AudioFrameModel, video::ImageFrameModel};
use serde::Deserialize;
use surrealdb::sql::Thing;

#[derive(Debug, Deserialize)]
pub struct AudioFrameEntity {
    id: Thing,
    data: Vec<TextEntity>,
    start_timestamp: i64,
    end_timestamp: i64,
}

impl From<AudioFrameEntity> for AudioFrameModel {
    fn from(value: AudioFrameEntity) -> Self {
        Self {
            id: Some(value.id.into()),
            start_timestamp: value.start_timestamp,
            end_timestamp: value.end_timestamp,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ImageFrameEntity {
    id: Thing,
    data: Vec<ImageEntity>,
    start_timestamp: i64,
    end_timestamp: i64,
}

impl From<ImageFrameEntity> for ImageFrameModel {
    fn from(value: ImageFrameEntity) -> Self {
        Self {
            id: Some(value.id.into()),
            start_timestamp: value.start_timestamp,
            end_timestamp: value.end_timestamp,
        }
    }
}
