use crate::image::ImageTaskType;
use crate::raw_text::RawTextTaskType;
use crate::web_page::WebPageTaskType;
use crate::{audio::AudioTaskType, video::VideoTaskType};
use content_base_derive::ContentTask;
use serde::{Deserialize, Serialize};
use std::hash::Hash;
use std::{fmt, path::PathBuf};
use storage_macro::Storage;

#[derive(Clone, Debug)]
pub struct FileInfo {
    /// Unique identifier for the file, which is the hash of the asset_object in database
    pub file_identifier: String,
    /// Full path to the file on disk
    pub file_full_path_on_disk: PathBuf,
}

#[derive(ContentTask, Clone, Debug, Storage)]
pub enum ContentTaskType {
    Video(VideoTaskType),
    Audio(AudioTaskType),
    Image(ImageTaskType),
    RawText(RawTextTaskType),
    WebPage(WebPageTaskType),
}

impl fmt::Display for ContentTaskType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ContentTaskType::Video(t) => write!(f, "video-{}", t),
            ContentTaskType::Audio(t) => write!(f, "audio-{}", t),
            ContentTaskType::Image(t) => write!(f, "image-{}", t),
            ContentTaskType::RawText(t) => write!(f, "raw-text-{}", t),
            ContentTaskType::WebPage(t) => write!(f, "web-page-{}", t),
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
            value if value.starts_with("raw-text-") => {
                RawTextTaskType::try_from(&value["raw-text-".len()..])
                    .map(|v| ContentTaskType::RawText(v))
                    .map_err(|e| anyhow::anyhow!(e))
            }
            value if value.starts_with("web-page-") => {
                WebPageTaskType::try_from(&value["web-page-".len()..])
                    .map(|v| ContentTaskType::WebPage(v))
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

#[allow(unused_imports, dead_code)]
mod test {
    use crate::video::frame::VideoFrameTask;
    use crate::video::VideoTaskType;
    use crate::ContentTaskType;

    #[test]
    fn test_display() {
        println!(
            "{}",
            ContentTaskType::Video(VideoTaskType::Frame(VideoFrameTask {}))
        );
    }
}
