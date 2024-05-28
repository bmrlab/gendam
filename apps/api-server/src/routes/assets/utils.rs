use blake3::Hasher;
use std::path::Path;
use tokio::{
    fs::{self, File},
    io::{self, AsyncReadExt, AsyncSeekExt, SeekFrom},
};

const SAMPLE_COUNT: u64 = 4;
const SAMPLE_SIZE: u64 = 1024 * 10;
const HEADER_OR_FOOTER_SIZE: u64 = 1024 * 8;
// minimum file size of 100KiB, to avoid sample hashing for small files as they can be smaller than the total sample size
const MINIMUM_FILE_SIZE: u64 = 1024 * 100;

// use static_assertions::const_assert;
// // Asserting that nobody messed up our consts
// const_assert!((HEADER_OR_FOOTER_SIZE * 2 + SAMPLE_COUNT * SAMPLE_SIZE) < MINIMUM_FILE_SIZE);
// // Asserting that the sample size is larger than header/footer size, as the same buffer is used for both
// const_assert!(SAMPLE_SIZE > HEADER_OR_FOOTER_SIZE);

pub async fn generate_file_hash(path: impl AsRef<Path>, size: u64) -> Result<String, io::Error> {
	let mut hasher = Hasher::new();
	hasher.update(&size.to_le_bytes());

	if size <= MINIMUM_FILE_SIZE {
		// For small files, we hash the whole file
		hasher.update(&fs::read(path).await?);
	} else {
		let mut file = File::open(path).await?;
		let mut buf = vec![0; SAMPLE_SIZE as usize].into_boxed_slice();

		// Hashing the header
		let mut current_pos = file
			.read_exact(&mut buf[..HEADER_OR_FOOTER_SIZE as usize])
			.await? as u64;
		hasher.update(&buf[..HEADER_OR_FOOTER_SIZE as usize]);

		// Sample hashing the inner content of the file
		let seek_jump = (size - HEADER_OR_FOOTER_SIZE * 2) / SAMPLE_COUNT;
		loop {
			file.read_exact(&mut buf).await?;
			hasher.update(&buf);

			if current_pos >= (HEADER_OR_FOOTER_SIZE + seek_jump * (SAMPLE_COUNT - 1)) {
				break;
			}

			current_pos = file.seek(SeekFrom::Start(current_pos + seek_jump)).await?;
		}

		// Hashing the footer
		file.seek(SeekFrom::End(-(HEADER_OR_FOOTER_SIZE as i64)))
			.await?;
		file.read_exact(&mut buf[..HEADER_OR_FOOTER_SIZE as usize])
			.await?;
		hasher.update(&buf[..HEADER_OR_FOOTER_SIZE as usize]);
	}

	Ok(hasher.finalize().to_hex()[..16].to_string())
}

// pub(crate) fn contains_invalid_chars(name: &str) -> bool {
//     name.chars().any(|c| match c {
//         '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => true,
//         _ => false,
//     })
// }

// pub(crate) fn normalized_materialized_path(path: &str) -> String {
//     if path.ends_with("/") {
//         path.to_string()
//     } else {
//         format!("{}/", path)
//     }
// }

pub enum FileType {
    Video,
    Image,
    Other,
}

pub fn get_file_type(mime_type: Option<String>) -> FileType {
    if let Some(mime_type) = mime_type {
        if mime_type.starts_with("video/") {
            return FileType::Video;
        } else if mime_type.starts_with("image/") {
            return FileType::Image;
        } else {
            return FileType::Other;
        }
    }
    FileType::Other
}
