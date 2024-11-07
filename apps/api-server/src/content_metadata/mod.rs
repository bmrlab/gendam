mod audio;
mod image;
mod raw_text;
mod video;
mod web_page;

use audio::AudioMetadata;
use content_base::ContentMetadata;
use image::ImageMetadata;
use raw_text::RawTextMetadata;
use serde::{Deserialize, Serialize};
use specta::Type;
use video::VideoMetadata;
use web_page::WebPageMetadata;

#[derive(Clone, Debug, Serialize, Deserialize, Type)]
#[serde(tag = "contentType")]
pub enum ContentMetadataWithType {
    Audio(AudioMetadata),
    Video(VideoMetadata),
    Image(ImageMetadata),
    RawText(RawTextMetadata),
    WebPage(WebPageMetadata),
    Unknown,
}

impl From<&ContentMetadata> for ContentMetadataWithType {
    fn from(metadata: &ContentMetadata) -> Self {
        match metadata {
            ContentMetadata::Audio(metadata) => ContentMetadataWithType::Audio(metadata.into()),
            ContentMetadata::Video(metadata) => ContentMetadataWithType::Video(metadata.into()),
            ContentMetadata::Image(metadata) => ContentMetadataWithType::Image(metadata.into()),
            ContentMetadata::RawText(metadata) => ContentMetadataWithType::RawText(metadata.into()),
            ContentMetadata::WebPage(metadata) => ContentMetadataWithType::WebPage(metadata.into()),
            ContentMetadata::Unknown => ContentMetadataWithType::Unknown,
            _ => {
                unreachable!("Unsupported metadata type")
            }
        }
    }
}
