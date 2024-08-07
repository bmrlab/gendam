use crate::audio::AudioDecoder;
use content_metadata::ContentMetadata;
use std::path::Path;

pub(crate) fn get_audio_metadata(file_path: impl AsRef<Path>) -> anyhow::Result<ContentMetadata> {
    let audio_decoder = AudioDecoder::new(file_path.as_ref())?;
    let metadata = audio_decoder.get_audio_metadata()?;
    Ok(ContentMetadata::Audio(metadata))
}
