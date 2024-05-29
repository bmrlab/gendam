use crate::{error::StorageResult, Storage};
use async_trait::async_trait;
use bytes::Bytes;
use opendal::Buffer;

#[async_trait]
pub trait StorageTrait {
    fn get_storage(&self) -> Storage;

    fn get_actual_path(&self, path: std::path::PathBuf) -> std::path::PathBuf;

    fn read_blocking(&self, path: std::path::PathBuf) -> StorageResult<Buffer>;

    fn read_to_string(&self, path: std::path::PathBuf) -> StorageResult<String>;

    fn write_blocking(&self, path: std::path::PathBuf, bs: Bytes) -> StorageResult<()>;

    fn remove_file(&self, path: std::path::PathBuf) -> StorageResult<()>;

    async fn create_dir(&self, path: std::path::PathBuf) -> StorageResult<()>;

    async fn is_exist(&self, path: std::path::PathBuf) -> StorageResult<bool>;

    async fn read(&self, path: std::path::PathBuf) -> StorageResult<Buffer>;

    async fn write(&self, path: std::path::PathBuf, bs: Buffer) -> StorageResult<()>;

    async fn copy(&self, from: std::path::PathBuf, to: std::path::PathBuf) -> StorageResult<()>;

    async fn read_dir(&self, path: std::path::PathBuf) -> StorageResult<Vec<std::path::PathBuf>>;

    async fn remove_dir_all(&self, path: std::path::PathBuf) -> StorageResult<()>;
}
