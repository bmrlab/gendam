use crate::db::model::id::ID;
use educe::Educe;
use serde::Serialize;

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
