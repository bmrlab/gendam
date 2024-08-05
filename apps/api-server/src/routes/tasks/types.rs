use content_base::{audio::AudioTaskType, video::VideoTaskType, ContentTaskType};
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Clone, Debug, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum VideoTaskTypeSpecta {
    Thumbnail,
    Frame,
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

#[derive(Clone, Debug, Type, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "contentType", content = "taskType")]
pub enum ContentTaskTypeSpecta {
    Video(VideoTaskTypeSpecta),
    Audio(AudioTaskTypeSpecta),
}

impl From<ContentTaskType> for ContentTaskTypeSpecta {
    fn from(task_type: ContentTaskType) -> Self {
        match task_type {
            ContentTaskType::Video(t) => match t {
                VideoTaskType::Thumbnail(_) => {
                    ContentTaskTypeSpecta::Video(VideoTaskTypeSpecta::Thumbnail)
                }
                VideoTaskType::Frame(_) => ContentTaskTypeSpecta::Video(VideoTaskTypeSpecta::Frame),
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
        }
    }
}
