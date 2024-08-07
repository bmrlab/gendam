mod audio;
mod constants;
mod image;
mod video;

use audio::get_audio_metadata;
use constants::{get_kind_from_extension, get_kind_from_mime, get_mime_from_extension};
use content_metadata::{ContentMetadata, ContentType};
use image::get_image_metadata;
use std::path::Path;
use video::get_video_metadata;

/// Extract file metadata from a file path. The file extension will be used firstly if provided explicitly.
/// Otherwise, magic number from the file will be used to determine the type.
pub fn file_metadata(
    file_path: impl AsRef<Path>,
    file_extension: Option<&str>,
) -> (ContentMetadata, String) {
    // if file extension is provided, use it
    if let Some(file_extension) = file_extension {
        let file_extension = file_extension.trim_start_matches('.').to_lowercase();
        let kind = get_kind_from_extension(&file_extension);

        if let Some(kind) = kind {
            if let Ok(metadata) = get_file_metadata(&kind, &file_path) {
                return (
                    metadata,
                    get_mime_from_extension(&file_extension)
                        .unwrap_or("")
                        .to_string(),
                );
            }
        }
    }

    if let Ok(Some(kind)) = infer::get_from_path(file_path.as_ref()) {
        if let Some(content_type) = get_kind_from_mime(kind.mime_type()) {
            if let Ok(metadata) = get_file_metadata(&content_type, &file_path) {
                return (metadata, kind.mime_type().to_string());
            }
        }
    }

    (ContentMetadata::Unknown, "".to_string())
}

fn get_file_metadata(
    content_type: &ContentType,
    file_path: impl AsRef<Path>,
) -> anyhow::Result<ContentMetadata> {
    match content_type {
        ContentType::Image => get_image_metadata(file_path),
        ContentType::Video => get_video_metadata(file_path),
        ContentType::Audio => get_audio_metadata(file_path),
        ContentType::RawText => {
            todo!()
        }
        _ => Ok(ContentMetadata::Unknown),
    }
}
