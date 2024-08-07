use crate::{audio::AudioDecoder, video::VideoDecoder};
use content_metadata::{image::ImageMetadata, ContentMetadata};
use image::ImageReader;
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
                _ if mime_type.starts_with("image") => {
                    let image = ImageReader::open(file_path.as_ref())?.with_guessed_format()?.decode()?;
                    Ok(ContentMetadata::Image(ImageMetadata {
                        width: image.width(),
                        height: image.height(),
                        color: match image.color() {
                            image::ColorType::L8 => "L",
                            image::ColorType::La8 => "LA",
                            image::ColorType::Rgb8 => "RGB",
                            image::ColorType::Rgba8 => "RGBA",

                            image::ColorType::L16 => "L",
                            image::ColorType::La16 => "LA",
                            image::ColorType::Rgb16 => "RGB",
                            image::ColorType::Rgba16 => "RGBA",

                            image::ColorType::Rgb32F => "RGB",
                            image::ColorType::Rgba32F => "RGBA",
                            _ => "",
                        }
                        .to_string(),
                    }))
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
