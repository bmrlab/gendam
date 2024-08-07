use std::path::Path;
use content_metadata::{image::ImageMetadata, ContentMetadata};
use image::ImageReader;

pub(crate) fn get_image_metadata(file_path: impl AsRef<Path>) -> anyhow::Result<ContentMetadata> {
    let image = ImageReader::open(file_path.as_ref())?
        .with_guessed_format()?
        .decode()?;
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
            _ => "Unknown",
        }
        .to_string(),
    }))
}
