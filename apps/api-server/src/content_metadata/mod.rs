mod audio;
mod image;
mod video;

use audio::AudioMetadata;
use content_base::ContentMetadata;
use image::ImageMetadata;
use serde::{Deserialize, Serialize};
use specta::Type;
use video::VideoMetadata;

#[derive(Clone, Debug, Serialize, Deserialize, Type)]
#[serde(tag = "contentType", rename_all = "camelCase")]
pub enum ContentMetadataWithType {
    Audio(AudioMetadata),
    Video(VideoMetadata),
    Image(ImageMetadata),
    Unknown,
}

impl From<&ContentMetadata> for ContentMetadataWithType {
    fn from(metadata: &ContentMetadata) -> Self {
        match metadata {
            ContentMetadata::Audio(metadata) => ContentMetadataWithType::Audio(metadata.into()),
            ContentMetadata::Video(metadata) => ContentMetadataWithType::Video(metadata.into()),
            ContentMetadata::Image(metadata) => ContentMetadataWithType::Image(metadata.into()),
            ContentMetadata::Unknown => ContentMetadataWithType::Unknown,
            _ => {
                todo!()
            }
        }
    }
}
