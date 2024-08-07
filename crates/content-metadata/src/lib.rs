pub mod audio;
pub mod html;
pub mod image;
pub mod raw_text;
pub mod video;

use audio::AudioMetadata;
use html::HTMLMetadata;
use image::ImageMetadata;
use raw_text::RawTextMetadata;
use serde::{Deserialize, Serialize};
use strum_macros::EnumDiscriminants;
use video::VideoMetadata;

#[derive(Clone, Debug, Serialize, Deserialize, EnumDiscriminants)]
#[strum_discriminants(derive(Serialize, Deserialize, strum_macros::Display))]
#[strum_discriminants(name(ContentType))]
#[serde(tag = "content_type")]
#[non_exhaustive]
pub enum ContentMetadata {
    Audio(AudioMetadata),
    Video(VideoMetadata),
    Image(ImageMetadata),
    RawText(RawTextMetadata),
    Html(HTMLMetadata),
    // Document(),
    // Presentation(),
    // Sheet(),
    // PDF(),
    Unknown,
}

impl Default for ContentMetadata {
    fn default() -> Self {
        Self::Unknown
    }
}
