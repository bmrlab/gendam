use crate::db::model::id::ID;
use crate::db::model::TextModel;
use educe::Educe;

#[derive(Educe)]
#[educe(Debug)]
pub struct AudioFrameModel {
    pub id: Option<ID>,

    #[educe(Debug(ignore))]
    pub data: Vec<TextModel>,
    pub start_timestamp: f32,
    pub end_timestamp: f32,
}

#[derive(Debug)]
pub struct AudioModel {
    pub id: Option<ID>,
    pub audio_frame: Vec<AudioFrameModel>,
}
