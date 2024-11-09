use crate::db::model::id::ID;
use educe::Educe;
use serde::Serialize;

#[derive(Serialize, Educe, Clone)]
#[educe(Debug)]
pub struct ImageModel {
    pub id: Option<ID>,
    pub prompt: String,

    #[educe(Debug(ignore))]
    pub vector: Vec<f32>,

    #[educe(Debug(ignore))]
    pub prompt_vector: Vec<f32>,
}
