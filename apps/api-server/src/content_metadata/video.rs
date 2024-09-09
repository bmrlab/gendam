use super::audio::AudioMetadata;
use content_base::metadata::video::{
    VideoAvgFrameRate as RawVideoAvgFrameRate, VideoMetadata as RawVideoMetadata,
};
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct VideoAvgFrameRate {
    pub numerator: String,
    pub denominator: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct VideoMetadata {
    pub width: String,
    pub height: String,
    /// video duration in seconds
    pub duration: f64,
    pub bit_rate: String,
    pub avg_frame_rate: VideoAvgFrameRate,
    pub audio: Option<AudioMetadata>,
}

impl From<&RawVideoAvgFrameRate> for VideoAvgFrameRate {
    fn from(rate: &RawVideoAvgFrameRate) -> Self {
        Self {
            numerator: rate.numerator.to_string(),
            denominator: rate.denominator.to_string(),
        }
    }
}

impl From<&RawVideoMetadata> for VideoMetadata {
    fn from(metadata: &RawVideoMetadata) -> Self {
        Self {
            width: metadata.width.to_string(),
            height: metadata.height.to_string(),
            duration: metadata.duration,
            bit_rate: metadata.bit_rate.to_string(),
            avg_frame_rate: VideoAvgFrameRate::from(&metadata.avg_frame_rate),
            audio: metadata.audio.as_ref().map(|v| AudioMetadata::from(v)),
        }
    }
}
