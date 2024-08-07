use crate::video::VideoDecoder;
use content_metadata::ContentMetadata;
use std::path::Path;

pub(crate) fn get_video_metadata(file_path: impl AsRef<Path>) -> anyhow::Result<ContentMetadata> {
    let video_decoder = VideoDecoder::new(file_path.as_ref())?;
    let metadata = video_decoder.get_video_metadata()?;
    Ok(ContentMetadata::Video(metadata))
}
