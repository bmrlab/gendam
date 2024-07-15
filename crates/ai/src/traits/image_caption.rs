use super::AIModel;
use std::path::PathBuf;

pub type ImageCaptionInput = PathBuf;
pub type ImageCaptionOutput = String;
pub type ImageCaptionModel = AIModel<ImageCaptionInput, ImageCaptionOutput>;
