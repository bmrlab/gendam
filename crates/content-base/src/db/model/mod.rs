use serde::{Deserialize, Serialize};
pub mod payload;
pub mod id;
pub mod audio;
pub mod video;
pub mod web;
pub mod document;

#[derive(Serialize, Deserialize)]
pub struct ImageModel {
    pub prompt: String,
    pub vector: Vec<f32>,
    pub prompt_vector: Vec<f32>,
}

#[derive(Serialize, Deserialize)]
pub struct TextModel {
    pub data: String,
    pub vector: Vec<f32>,
    pub en_data: String,
    pub en_vector: Vec<f32>,
}

#[derive(Serialize, Deserialize)]
pub struct PageModel {
    pub text: Vec<TextModel>,
    pub image: Vec<ImageModel>,
    pub start_index: i32,
    pub end_index: i32,
}