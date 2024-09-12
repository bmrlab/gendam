use crate::db::model::TextModel;

#[derive(Debug)]

pub struct AudioFrameModel {
    pub data: Vec<TextModel>,
    pub start_timestamp: f32,
    pub end_timestamp: f32,
}

#[derive(Debug)]
pub struct AudioModel {
    pub audio_frame: Vec<AudioFrameModel>,
}