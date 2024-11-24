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

#[cfg_attr(feature = "rspc", derive(specta::Type))]
#[derive(Clone, Debug, Serialize, Deserialize, EnumDiscriminants)]
#[strum_discriminants(derive(Serialize, Deserialize, strum_macros::Display))]
#[strum_discriminants(name(ContentType))] // 这个宏会生成一个名为 ContentType 的辅助枚举类型
#[serde(tag = "contentType")] // 用于 serialize 了以后写入数据库 assetObject.mediaData 里面的字段名
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
