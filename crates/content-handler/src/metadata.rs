use crate::{audio::AudioDecoder, video::VideoDecoder};
use content_metadata::ContentMetadata;
use std::path::Path;

pub fn file_metadata(file_path: impl AsRef<Path>) -> anyhow::Result<ContentMetadata> {
    let kind = infer::get_from_path(file_path.as_ref())?;

    match kind {
        Some(kind) => {
            let mime_type = kind.mime_type();
            match mime_type {
                _ if mime_type.starts_with("video") => {
                    let video_decoder = VideoDecoder::new(file_path.as_ref())?;
                    let metadata = video_decoder.get_video_metadata()?;
                    Ok(ContentMetadata::Video(metadata))
                }
                _ if mime_type.starts_with("audio") => {
                    let audio_decoder = AudioDecoder::new(file_path.as_ref())?;
                    let metadata = audio_decoder.get_audio_metadata()?;
                    Ok(ContentMetadata::Audio(metadata))
                }
                _ => {
                    tracing::warn!("unsupported mime type: {}", mime_type);
                    Ok(ContentMetadata::Unknown)
                }
            }
        }
        _ => Ok(ContentMetadata::Unknown),
    }
}
