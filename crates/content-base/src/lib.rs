pub mod artifacts;
mod audio;
pub mod constants;
mod core;
mod handler;
pub mod metadata;
mod payload;
pub mod record;
mod traits;
mod video;

use ::core::fmt;
use audio::AudioTaskType;
use content_base_derive::ContentTask;
use storage_macro::Storage;
pub use core::*;
use serde::{Deserialize, Serialize};
use std::hash::Hash;
pub use traits::*;
use video::VideoTaskType;
use async_trait::async_trait;

#[derive(ContentTask, Clone, Debug, Storage)]
pub enum ContentTaskType {
    Video(VideoTaskType),
    Audio(AudioTaskType),
}

impl fmt::Display for ContentTaskType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ContentTaskType::Video(t) => write!(f, "video-{}", t),
            ContentTaskType::Audio(t) => write!(f, "audio-{}", t),
        }
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
