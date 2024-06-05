use crate::{traits::Storage, utils::path_to_string, StorageResult};
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
        let root = path_to_string(root)?;
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
}
