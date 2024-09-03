use crate::db::model::audio::AudioFrameModel;
use crate::db::model::ImageModel;

pub struct ImageFrameModel {
    pub data: Vec<ImageModel>,
    pub start_timestamp: f32,
    pub end_timestamp: f32,
}

pub struct VideoModel {
    pub image_frame: Vec<ImageFrameModel>,
    pub audio_frame: Vec<AudioFrameModel>,
}