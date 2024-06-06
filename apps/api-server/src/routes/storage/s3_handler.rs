use std::path::PathBuf;

use global_variable::{current_library_dir, get_current_s3_storage};
use storage::prelude::*;

fn dir_in_file(hash: &str) -> PathBuf {
    PathBuf::from(current_library_dir!())
        .join("files")
        .join(&hash[0..3])
}

fn dir_in_artifacts(hash: &str) -> PathBuf {
    PathBuf::from(current_library_dir!())
        .join("artifacts")
        .join(&hash[0..3])
        .join(hash)
}

pub async fn upload_to_s3(hash: String) -> StorageResult<()> {
    let storage = get_current_s3_storage!()?;

    let file_path = dir_in_file(hash.as_str());
    let artifact_path = dir_in_artifacts(hash.as_str());

    storage.upload_dir_recursive(file_path).await?;
    storage.upload_dir_recursive(artifact_path).await?;

    Ok(())
}
