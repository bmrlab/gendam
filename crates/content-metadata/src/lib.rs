pub mod audio;
pub mod image;
pub mod video;

use audio::AudioMetadata;
use image::ImageMetadata;
use serde::{Deserialize, Serialize};
use strum_macros::EnumDiscriminants;
use video::VideoMetadata;

#[derive(Clone, Debug, Serialize, Deserialize, EnumDiscriminants)]
#[strum_discriminants(derive(Serialize, Deserialize, strum_macros::Display))]
#[strum_discriminants(name(ContentType))]
#[serde(tag = "content_type")]
pub enum ContentMetadata {
    Audio(AudioMetadata),
    Video(VideoMetadata),
    Image(ImageMetadata),
    Unknown,
}

impl Default for ContentMetadata {
    fn default() -> Self {
        Self::Unknown
    }
}
