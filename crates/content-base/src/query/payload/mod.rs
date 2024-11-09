pub mod audio;
pub mod image;
pub mod raw_text;
pub mod video;
pub mod web_page;

use self::{
    audio::AudioIndexMetadata, image::ImageIndexMetadata, raw_text::RawTextIndexMetadata,
    video::VideoIndexMetadata, web_page::WebPageIndexMetadata,
};
use serde::Serialize;

// ContentIndexMetadata uses tagged variant serialization
// When serialized to JSON, variants will include a "content_type" field indicating the variant type
// {
//   "contentType": "Video",
//   ...VideoIndexMetadata fields...
// }
// 这个对象的数据在 expand_hit_result 函数中被计算出来
#[cfg_attr(feature = "rspc", derive(specta::Type))]
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "contentType")]
pub enum ContentIndexMetadata {
    Video(VideoIndexMetadata),
    Audio(AudioIndexMetadata),
    Image(ImageIndexMetadata),
    RawText(RawTextIndexMetadata),
    WebPage(WebPageIndexMetadata),
}

#[derive(Debug, Serialize)]
pub struct ContentQueryResult {
    pub file_identifier: String,
    pub score: f32,
    pub metadata: ContentIndexMetadata,
    pub hit_text: Option<String>,          // 命中的索引内容
    pub reference_content: Option<String>, // 根据 metadata 提取出来的内容片段
}

// #[derive(Debug, Serialize)]
// pub struct SearchRequest {
//     pub text: String,
// }
