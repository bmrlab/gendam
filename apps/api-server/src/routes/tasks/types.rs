use content_base::{
    audio::AudioTaskType, image::ImageTaskType, raw_text::RawTextTaskType, video::VideoTaskType,
    web_page::WebPageTaskType, ContentTaskType,
};
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Clone, Debug, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum VideoTaskTypeSpecta {
    Thumbnail,
    Frame,
    FrameEmbedding,
    FrameDescription,
    FrameDescEmbed,
    Audio,
    Transcript,
    TransChunk,
    TransChunkSum,
    TransChunkSumEmbed,
}

#[derive(Clone, Debug, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum AudioTaskTypeSpecta {
    Thumbnail,
    Waveform,
    Transcript,
    TransChunk,
    TransChunkSum,
    TransChunkSumEmbed,
}

#[derive(Clone, Debug, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum ImageTaskTypeSpecta {
    Thumbnail,
    Embedding,
    Description,
    DescEmbed,
}

#[derive(Clone, Debug, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum RawTextTaskTypeSpecta {
    Chunk,
    ChunkSum,
    ChunkSumEmbed,
}

#[derive(Clone, Debug, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum WebPageTaskTypeSpecta {
    Transform,
    Chunk,
    ChunkSum,
    ChunkSumEmbed,
}

#[derive(Clone, Debug, Type, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "contentType", content = "taskType")]
pub enum ContentTaskTypeSpecta {
    Video(VideoTaskTypeSpecta),
    Audio(AudioTaskTypeSpecta),
    Image(ImageTaskTypeSpecta),
    RawText(RawTextTaskTypeSpecta),
    WebPage(WebPageTaskTypeSpecta),
}

impl From<ContentTaskType> for ContentTaskTypeSpecta {
    fn from(task_type: ContentTaskType) -> Self {
        match task_type {
            ContentTaskType::Video(t) => match t {
                VideoTaskType::Thumbnail(_) => {
                    ContentTaskTypeSpecta::Video(VideoTaskTypeSpecta::Thumbnail)
                }
                VideoTaskType::Frame(_) => ContentTaskTypeSpecta::Video(VideoTaskTypeSpecta::Frame),
                VideoTaskType::FrameEmbedding(_) => {
                    ContentTaskTypeSpecta::Video(VideoTaskTypeSpecta::FrameEmbedding)
                }
                VideoTaskType::FrameDescription(_) => {
                    ContentTaskTypeSpecta::Video(VideoTaskTypeSpecta::FrameDescription)
                }
                VideoTaskType::FrameDescEmbed(_) => {
                    ContentTaskTypeSpecta::Video(VideoTaskTypeSpecta::FrameDescEmbed)
                }
                VideoTaskType::Audio(_) => ContentTaskTypeSpecta::Video(VideoTaskTypeSpecta::Audio),
                VideoTaskType::Transcript(_) => {
                    ContentTaskTypeSpecta::Video(VideoTaskTypeSpecta::Transcript)
                }
                VideoTaskType::TransChunk(_) => {
                    ContentTaskTypeSpecta::Video(VideoTaskTypeSpecta::TransChunk)
                }
                VideoTaskType::TransChunkSum(_) => {
                    ContentTaskTypeSpecta::Video(VideoTaskTypeSpecta::TransChunkSum)
                }
                VideoTaskType::TransChunkSumEmbed(_) => {
                    ContentTaskTypeSpecta::Video(VideoTaskTypeSpecta::TransChunkSumEmbed)
                }
            },
            ContentTaskType::Audio(t) => match t {
                AudioTaskType::Thumbnail(_) => {
                    ContentTaskTypeSpecta::Audio(AudioTaskTypeSpecta::Thumbnail)
                }
                AudioTaskType::Waveform(_) => {
                    ContentTaskTypeSpecta::Audio(AudioTaskTypeSpecta::Waveform)
                }
                AudioTaskType::Transcript(_) => {
                    ContentTaskTypeSpecta::Audio(AudioTaskTypeSpecta::Transcript)
                }
                AudioTaskType::TransChunk(_) => {
                    ContentTaskTypeSpecta::Audio(AudioTaskTypeSpecta::TransChunk)
                }
                AudioTaskType::TransChunkSum(_) => {
                    ContentTaskTypeSpecta::Audio(AudioTaskTypeSpecta::TransChunkSum)
                }
                AudioTaskType::TransChunkSumEmbed(_) => {
                    ContentTaskTypeSpecta::Audio(AudioTaskTypeSpecta::TransChunkSumEmbed)
                }
            },
            ContentTaskType::Image(t) => match t {
                ImageTaskType::Thumbnail(_) => {
                    ContentTaskTypeSpecta::Image(ImageTaskTypeSpecta::Thumbnail)
                }
                ImageTaskType::Embedding(_) => {
                    ContentTaskTypeSpecta::Image(ImageTaskTypeSpecta::Embedding)
                }
                ImageTaskType::Description(_) => {
                    ContentTaskTypeSpecta::Image(ImageTaskTypeSpecta::Description)
                }
                ImageTaskType::DescEmbed(_) => {
                    ContentTaskTypeSpecta::Image(ImageTaskTypeSpecta::DescEmbed)
                }
            },
            ContentTaskType::RawText(t) => match t {
                RawTextTaskType::Chunk(_) => {
                    ContentTaskTypeSpecta::RawText(RawTextTaskTypeSpecta::Chunk)
                }
                RawTextTaskType::ChunkSum(_) => {
                    ContentTaskTypeSpecta::RawText(RawTextTaskTypeSpecta::ChunkSum)
                }
                RawTextTaskType::ChunkSumEmbed(_) => {
                    ContentTaskTypeSpecta::RawText(RawTextTaskTypeSpecta::ChunkSumEmbed)
                }
            },
            ContentTaskType::WebPage(t) => match t {
                WebPageTaskType::Transform(_) => {
                    ContentTaskTypeSpecta::WebPage(WebPageTaskTypeSpecta::Transform)
                }
                WebPageTaskType::Chunk(_) => {
                    ContentTaskTypeSpecta::WebPage(WebPageTaskTypeSpecta::Chunk)
                }
                WebPageTaskType::ChunkSum(_) => {
                    ContentTaskTypeSpecta::WebPage(WebPageTaskTypeSpecta::ChunkSum)
                }
                WebPageTaskType::ChunkSumEmbed(_) => {
                    ContentTaskTypeSpecta::WebPage(WebPageTaskTypeSpecta::ChunkSumEmbed)
                }
            },
        }
    }
}
