use educe::Educe;
use crate::db::model::audio::AudioFrameModel;
use crate::db::model::id::ID;
use crate::db::model::ImageModel;

#[derive(Educe)]
#[educe(Debug)]
pub struct ImageFrameModel {
    pub id: Option<ID>,
    
    #[educe(Debug(ignore))]
    pub data: Vec<ImageModel>,
    pub start_timestamp: f32,
    pub end_timestamp: f32,
}

#[derive(Educe)]
#[educe(Debug)]
pub struct VideoModel {
    pub id: Option<ID>,
    
    #[educe(Debug(ignore))]
    pub image_frame: Vec<ImageFrameModel>,
    
    #[educe(Debug(ignore))]
    pub audio_frame: Vec<AudioFrameModel>,
}
