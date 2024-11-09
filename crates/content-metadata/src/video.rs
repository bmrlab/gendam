use super::audio::AudioMetadata;
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "rspc", derive(specta::Type))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")] // 用于 serialize 了以后写入数据库 assetObject.mediaData 里面的字段名
pub struct VideoAvgFrameRate {
    #[cfg_attr(feature = "rspc", specta(type = u32))]
    pub numerator: usize,
    #[cfg_attr(feature = "rspc", specta(type = u32))]
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

#[cfg_attr(feature = "rspc", derive(specta::Type))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")] // 用于 serialize 了以后写入数据库 assetObject.mediaData 里面的字段名
pub struct VideoMetadata {
    #[cfg_attr(feature = "rspc", specta(type = u32))]
    pub width: usize,
    #[cfg_attr(feature = "rspc", specta(type = u32))]
    pub height: usize,
    /// video duration in seconds
    #[cfg_attr(feature = "rspc", specta(type = u32))]
    pub duration: f64,
    #[cfg_attr(feature = "rspc", specta(type = u32))]
    pub bit_rate: usize,
    pub avg_frame_rate: VideoAvgFrameRate,
    pub audio: Option<AudioMetadata>,
}

impl VideoMetadata {
    pub fn with_audio(&mut self, metadata: AudioMetadata) {
        self.audio = Some(metadata);
    }
}
