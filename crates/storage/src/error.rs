use opendal::Error as OpenDalError;
use thiserror::Error;

pub type StorageResult<T> = std::result::Result<T, StorageError>;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Storage error: {0}")]
    OpenDalError(#[from] OpenDalError),

    #[error("Storage unexpected error")]
    UnexpectedError,

    #[error("Storage tokio fs error: {0}")]
    TokioFsError(#[from] tokio::io::Error),

    #[error("UTF-8 conversion error: {0}")]
    Utf8Error(#[from] std::string::FromUtf8Error),

    #[error("Path error")]
    PathError,
}
