pub mod audio;
pub mod video;

use crate::{ContentBase, FileInfo};
use audio::AudioMetadata;
use file_handler::video::decoder::VideoDecoder;
use serde::{Deserialize, Serialize};
use strum::EnumDiscriminants;
use video::{VideoAvgFrameRate, VideoMetadata};

#[derive(Clone, Debug, Serialize, Deserialize, EnumDiscriminants)]
#[strum_discriminants(derive(Serialize, Deserialize, strum_macros::Display))]
#[strum_discriminants(name(ContentType))]
#[serde(tag = "content_type")]
pub enum ContentMetadata {
    Audio(AudioMetadata),
    Video(VideoMetadata),
    Unknown,
}

impl Default for ContentMetadata {
    fn default() -> Self {
        Self::Unknown
    }
}

impl ContentMetadata {
    pub async fn extract(
        file_info: &FileInfo,
        content_type: ContentType,
        ctx: &ContentBase,
    ) -> anyhow::Result<Self> {
        let metadata = match content_type {
            ContentType::Audio => {
                todo!()
            }
            ContentType::Video => {
                let video_decoder = VideoDecoder::new(&file_info.file_path)?;
                let metadata = video_decoder.get_video_metadata()?;

                Ok(Self::Video(VideoMetadata {
                    width: metadata.width,
                    height: metadata.height,
                    duration: metadata.duration,
                    bit_rate: metadata.bit_rate,
                    avg_frame_rate: VideoAvgFrameRate {
                        numerator: metadata.avg_frame_rate.numerator,
                        denominator: metadata.avg_frame_rate.denominator,
                    },
                    audio: metadata.audio.map(|v| AudioMetadata {
                        bit_rate: v.bit_rate,
                        duration: v.duration,
                    }),
                }))
            }
            ContentType::Unknown => Ok(Self::Unknown),
        };

        // write into artifacts
        if let Ok(metadata) = &metadata {
            ctx.set_metadata(&file_info.file_identifier, metadata).await?;
        }

        metadata
    }
}
