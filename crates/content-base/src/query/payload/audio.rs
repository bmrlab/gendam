use super::ContentIndexMetadata;
use serde::Serialize;

#[cfg_attr(feature = "rspc", derive(specta::Type))]
#[derive(Debug, Clone, Serialize)]
pub enum AudioSliceType {
    Transcript, // 语音转写，目前暂时只有这一个
}

#[cfg_attr(feature = "rspc", derive(specta::Type))]
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioIndexMetadata {
    pub slice_type: AudioSliceType,
    #[cfg_attr(feature = "rspc", specta(type = u32))]
    pub start_timestamp: i64,
    #[cfg_attr(feature = "rspc", specta(type = u32))]
    pub end_timestamp: i64,
}

impl TryFrom<ContentIndexMetadata> for AudioIndexMetadata {
    type Error = anyhow::Error;

    fn try_from(metadata: ContentIndexMetadata) -> Result<Self, Self::Error> {
        match metadata {
            ContentIndexMetadata::Audio(metadata) => Ok(metadata),
            _ => anyhow::bail!("metadata is not from audio"),
        }
    }
}

impl From<AudioIndexMetadata> for ContentIndexMetadata {
    fn from(metadata: AudioIndexMetadata) -> Self {
        ContentIndexMetadata::Audio(metadata)
    }
}

impl PartialEq for AudioIndexMetadata {
    fn eq(&self, other: &Self) -> bool {
        self.start_timestamp == other.start_timestamp && self.end_timestamp == other.end_timestamp
    }
}

impl Eq for AudioIndexMetadata {}

impl PartialOrd for AudioIndexMetadata {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.start_timestamp.partial_cmp(&other.start_timestamp) {
            Some(std::cmp::Ordering::Equal) => self.end_timestamp.partial_cmp(&other.end_timestamp),
            other => other,
        }
    }
}

impl Ord for AudioIndexMetadata {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}
