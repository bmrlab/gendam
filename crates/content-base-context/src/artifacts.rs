use storage::Storage;

use crate::ContentBaseCtx;
use std::path::PathBuf;

fn get_shard_hex(hash: &str) -> &str {
    &hash[0..3]
}

impl ContentBaseCtx {
    pub fn artifacts_dir(&self, file_identifier: &str) -> PathBuf {
        self.artifacts_dir
            .join(get_shard_hex(file_identifier))
            .join(file_identifier)
    }

    /// Delete all artifacts, this is not recommended to use.
    pub async fn delete_artifacts(&self, file_identifier: &str) -> anyhow::Result<()> {
        self.remove_dir_all(self.artifacts_dir(file_identifier))
            .await
            .map_err(|e| {
                anyhow::anyhow!("Failed to delete artifacts for {}: {}", file_identifier, e)
            })
    }
}
