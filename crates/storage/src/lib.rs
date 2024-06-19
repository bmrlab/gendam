mod config;
pub mod error;
mod traits;

pub use crate::error::StorageError;
pub use opendal::EntryMode;
pub mod utils;
pub use bytes::Bytes;
pub use error::StorageResult;
pub use opendal::Buffer;
pub use opendal::Metakey;
pub mod services;
pub use config::S3Config;
pub use opendal::BlockingOperator;
pub use opendal::Operator;
pub use traits::Storage;

pub use services::fs_storage::FsStorage;
pub use services::s3_storage::S3Storage;

pub mod prelude {
    pub use crate::FsStorage;
    pub use crate::S3Storage;
    pub use crate::Storage;
    pub use crate::StorageError;
    pub use crate::StorageResult;
}
