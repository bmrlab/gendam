use crate::image::ImageTaskType;
use crate::{audio::AudioTaskType, video::VideoTaskType};
use content_base_derive::ContentTask;
use serde::{Deserialize, Serialize};
use std::hash::Hash;
use std::{fmt, path::PathBuf};
use storage::Storage;
use storage_macro::Storage;

#[derive(Clone, Debug)]
pub struct FileInfo {
    pub file_identifier: String,
    pub file_path: PathBuf,
}

#[derive(ContentTask, Clone, Debug, Storage)]
pub enum ContentTaskType {
    Video(VideoTaskType),
    Audio(AudioTaskType),
    Image(ImageTaskType),
}

impl fmt::Display for ContentTaskType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ContentTaskType::Video(t) => write!(f, "video-{}", t),
            ContentTaskType::Audio(t) => write!(f, "audio-{}", t),
            ContentTaskType::Image(t) => write!(f, "image-{}", t),
        }
    }
}

impl From<&ContentTaskType> for ContentTaskType {
    fn from(value: &ContentTaskType) -> Self {
        value.clone()
    }
}

impl TryFrom<&str> for ContentTaskType {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            value if value.starts_with("video-") => {
                VideoTaskType::try_from(&value["video-".len()..])
                    .map(|v| ContentTaskType::Video(v))
                    .map_err(|e| anyhow::anyhow!(e))
            }
            value if value.starts_with("audio-") => {
                AudioTaskType::try_from(&value["audio-".len()..])
                    .map(|v| ContentTaskType::Audio(v))
                    .map_err(|e| anyhow::anyhow!(e))
            }
            value if value.starts_with("image-") => {
                ImageTaskType::try_from(&value["image-".len()..])
                    .map(|v| ContentTaskType::Image(v))
                    .map_err(|e| anyhow::anyhow!(e))
            }
            _ => Err(anyhow::anyhow!("invalid task type: {}", value)),
        }
    }
}

impl Serialize for ContentTaskType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for ContentTaskType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        ContentTaskType::try_from(s.as_str()).map_err(serde::de::Error::custom)
    }
}

impl PartialEq for ContentTaskType {
    fn eq(&self, other: &Self) -> bool {
        self.to_string() == other.to_string()
    }
}

impl Eq for ContentTaskType {}

impl Hash for ContentTaskType {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.to_string().hash(state);
    }
}
