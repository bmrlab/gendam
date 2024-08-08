pub mod audio;
pub mod image;
pub mod raw_text;
pub mod video;
pub mod web_page;

use audio::AudioMetadata;
use image::ImageMetadata;
use raw_text::RawTextMetadata;
use serde::{Deserialize, Serialize};
use strum_macros::EnumDiscriminants;
use video::VideoMetadata;
use web_page::WebPageMetadata;

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
    WebPage(WebPageMetadata),
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
