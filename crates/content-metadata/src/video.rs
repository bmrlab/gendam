use super::audio::AudioMetadata;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoAvgFrameRate {
    pub numerator: usize,
    pub denominator: usize,
}

impl From<String> for VideoAvgFrameRate {
    fn from(s: String) -> Self {
        let (numerator, denominator) = s
            .split_once('/')
            .map(|(numerator, denominator)| (numerator, denominator))
            .unwrap_or((s.as_str(), "1"));
        Self {
            numerator: numerator.parse().unwrap_or(0),
            denominator: denominator.parse().unwrap_or(1),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoMetadata {
    pub width: usize,
    pub height: usize,
    /// video duration in seconds
    pub duration: f64,
    pub bit_rate: usize,
    pub avg_frame_rate: VideoAvgFrameRate,
    pub audio: Option<AudioMetadata>,
}

impl VideoMetadata {
    pub fn with_audio(&mut self, metadata: AudioMetadata) {
        self.audio = Some(metadata);
    }
}
