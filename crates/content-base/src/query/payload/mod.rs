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
pub struct SearchResultData {
    pub file_identifier: String,
    pub score: f32,
    pub metadata: ContentIndexMetadata,
    pub highlight: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RetrievalResultData {
    pub file_identifier: String,
    pub score: f32,
    pub metadata: ContentIndexMetadata,
    pub reference_content: String, // 检索到的相关内容片段
}

#[derive(Debug, Serialize)]
pub struct SearchRequest {
    pub text: String,
}
