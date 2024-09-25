use crate::db::model::audio::AudioFrameModel;
use crate::db::model::id::ID;
use crate::db::model::ImageModel;

#[derive(Debug)]
pub struct ImageFrameModel {
    pub id: Option<ID>,
    pub data: Vec<ImageModel>,
    pub start_timestamp: f32,
    pub end_timestamp: f32,
}

#[derive(Debug)]
pub struct VideoModel {
    pub id: Option<ID>,
    pub image_frame: Vec<ImageFrameModel>,
    pub audio_frame: Vec<AudioFrameModel>,
}
