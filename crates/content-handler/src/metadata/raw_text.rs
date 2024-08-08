use content_metadata::{raw_text::RawTextMetadata, ContentMetadata};
use std::{fs::File, io::Read, path::Path};

pub(crate) fn get_raw_text_metadata(
    file_path: impl AsRef<Path>,
) -> anyhow::Result<ContentMetadata> {
    let mut file = File::open(file_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    Ok(ContentMetadata::RawText(RawTextMetadata {
        text_count: contents.chars().count() as u64,
    }))
}
