pub mod error;
mod traits;

pub use crate::error::StorageError;
pub mod utils;
pub use bytes::Bytes;
pub use error::StorageResult;
pub use opendal::Buffer;
pub use opendal::Metakey;
pub mod services;
pub use opendal::BlockingOperator;
pub use opendal::Operator;
pub use traits::Storage;

pub use services::fs_storage::FsStorage;
pub use services::s3_storage::S3Storage;
