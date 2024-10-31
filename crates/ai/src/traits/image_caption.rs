use super::AIModel;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct ImageCaptionInput {
    pub image_file_paths: Vec<PathBuf>,
    pub prompt: Option<String>,
}
// pub type ImageCaptionInput = PathBuf;
pub type ImageCaptionOutput = String;
pub type ImageCaptionModel = AIModel<ImageCaptionInput, ImageCaptionOutput>;
