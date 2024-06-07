use crate::{traits::Storage, utils::path_to_string, StorageError, StorageResult};
use async_trait::async_trait;
use opendal::{services::S3, BlockingOperator, Operator};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct S3Storage {
    root: PathBuf,
    op: Operator,
    block_op: BlockingOperator,
}

impl S3Storage {
    pub fn new(root: impl AsRef<Path>) -> StorageResult<Self> {
        let mut root = path_to_string(root)?;
        if !root.starts_with("/") {
            root = format!("/{}", root);
        }
        // Create s3 backend builder.
        let mut builder = S3::default();
        // Set the root for s3, all operations will happen under this root.
        //
        // NOTE: the root must be absolute path.
        builder.root(&root);
        // Set the bucket name. This is required.
        // TODO: replace with real config
        builder.bucket("my-test-bucket-131");
        builder.endpoint("http://127.0.0.1:9000");
        builder.access_key_id("plEXyNod8DWttxmCt3Db");
        builder.secret_access_key("IuJYIdJIdJm8LWQgCXP7af9pmis0dz4soEs7vp0U");
        let op: Operator = Operator::new(builder)?.finish();

        // Create s3 backend builder.
        let mut builder = S3::default();
        // Set the root for s3, all operations will happen under this root.
        //
        // NOTE: the root must be absolute path.
        builder.root(&root);
        // Set the bucket name. This is required.
        builder.bucket("my-test-bucket-131");
        builder.endpoint("http://127.0.0.1:9000");
        builder.access_key_id("plEXyNod8DWttxmCt3Db");
        builder.secret_access_key("IuJYIdJIdJm8LWQgCXP7af9pmis0dz4soEs7vp0U");
        builder.server_side_encryption_with_s3_key();
        let block_op = Operator::new(builder)?.finish().blocking();

        Ok(Self {
            op,
            block_op,
            root: PathBuf::from(root),
        })
    }
}

#[async_trait]
impl Storage for S3Storage {
    fn root(&self) -> StorageResult<PathBuf> {
        Ok(self.root.clone())
    }

    fn op(&self) -> StorageResult<Operator> {
        Ok(self.op.clone())
    }

    fn block_op(&self) -> StorageResult<BlockingOperator> {
        Ok(self.block_op.clone())
    }

    async fn upload_dir_recursive(
        &self,
        // relative path to root path
        dir: std::path::PathBuf,
    ) -> StorageResult<()> {
        for entry in std::fs::read_dir(dir)? {
            let entry_path = entry?.path();
            if entry_path.is_dir() {
                self.upload_dir_recursive(entry_path).await?;
            } else {
                let data = tokio::fs::read(&entry_path).await?;
                let components = entry_path.components();
                let path: PathBuf = components
                    .skip_while(|c| c.as_os_str() != "artifacts" && c.as_os_str() != "files")
                    .collect();
                self.write(path, data.into()).await?;
            }
        }
        Ok(())
    }
}
