use crate::ContentBase;
use std::path::PathBuf;

fn get_shard_hex(hash: &str) -> &str {
    &hash[0..3]
}

impl ContentBase {
    pub fn artifacts_dir(&self, file_identifier: &str) -> PathBuf {
        self.artifacts_dir
            .join(get_shard_hex(file_identifier))
            .join(file_identifier)
    }

    pub fn tmp_dir(&self, file_identifier: &str) -> PathBuf {
        self.tmp_dir
            .join(get_shard_hex(file_identifier))
            .join(file_identifier)
    }
}
