use content_base::metadata::audio::AudioMetadata as RawAudioMetadata;
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AudioMetadata {
    pub bit_rate: String,
    pub duration: f64,
}

impl From<&RawAudioMetadata> for AudioMetadata {
    fn from(metadata: &RawAudioMetadata) -> Self {
        Self {
            bit_rate: metadata.bit_rate.to_string(),
            duration: metadata.duration,
        }
    }
}
