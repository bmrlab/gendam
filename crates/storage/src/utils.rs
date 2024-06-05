use std::path::Path;

use crate::{StorageError, StorageResult};

#[macro_export]
macro_rules! add_tmp_suffix_to_path {
    ($path:expr) => {{
        let mut new_path = $path.to_path_buf();
        if let Some(file_stem) = new_path.file_stem() {
            let new_file_stem = format!("{}-tmp", file_stem.to_string_lossy());
            new_path.set_file_name(format!(
                "{}{}{}",
                new_file_stem,
                if new_path.extension().is_some() {
                    "."
                } else {
                    ""
                },
                new_path.extension().unwrap_or_default().to_string_lossy()
            ));
        }
        new_path
    }};
}

pub fn path_to_string(path: impl AsRef<Path>) -> StorageResult<String> {
    match path.as_ref().to_str() {
        Some(path) => Ok(path.to_string()),
        None => Err(StorageError::PathError),
    }
}
