use serde::{Deserialize, Serialize};
pub mod payload;
pub mod id;

#[derive(Serialize, Deserialize)]
pub struct ImageModel {
    pub prompt: String,
    pub vector: Vec<f32>,
    pub prompt_vector: Vec<f32>,
}

#[derive(Serialize, Deserialize)]
pub struct TextModel {
    pub vector: Vec<f32>,
    pub en_data: String,
    pub en_vector: Vec<f32>,
}

#[derive(Serialize, Deserialize)]
pub enum DataModel {
    Text(TextModel),
    Image(ImageModel),
}
