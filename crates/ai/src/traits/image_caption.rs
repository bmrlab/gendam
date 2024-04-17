use super::{AIModelLoader, AIModelTx};
use std::path::PathBuf;

pub type ImageCaptionInput = PathBuf;
pub type ImageCaptionOutput = String;

pub trait AsImageCaptionModel: Send + Sync {
    fn get_images_caption_tx(&self) -> AIModelTx<ImageCaptionInput, ImageCaptionOutput>;
}

impl AsImageCaptionModel for AIModelLoader<ImageCaptionInput, ImageCaptionOutput> {
    fn get_images_caption_tx(&self) -> AIModelTx<ImageCaptionInput, ImageCaptionOutput> {
        self.tx.clone()
    }
}
