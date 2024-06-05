use std::path::PathBuf;

use crate::{error::StorageResult, utils::path_to_string, StorageError};
use async_trait::async_trait;
use bytes::Bytes;
use opendal::{BlockingOperator, Buffer, Operator};

#[async_trait]
pub trait Storage {
    fn root(&self) -> StorageResult<std::path::PathBuf>;

    fn op(&self) -> StorageResult<Operator>;

    fn block_op(&self) -> StorageResult<BlockingOperator>;

    fn get_actual_path(&self, path: std::path::PathBuf) -> StorageResult<std::path::PathBuf> {
        Ok(self.root()?.join(path))
    }

    fn under_root(&self, path: std::path::PathBuf) -> StorageResult<bool> {
        Ok(path.is_relative())
    }

    fn read_blocking(&self, path: std::path::PathBuf) -> StorageResult<Buffer> {
        let path: String = path_to_string(path)?;
        self.block_op()?
            .read(path.as_str())
            .map_err(StorageError::from)
    }

    fn read_to_string(&self, path: std::path::PathBuf) -> StorageResult<String> {
        let path = path_to_string(path)?;
        self.block_op()?
            .read(path.as_str())
            .map(|bs| String::from_utf8(bs.to_vec()).map_err(StorageError::from))?
            .map_err(StorageError::from)
    }

    fn write_blocking(&self, path: std::path::PathBuf, bs: Bytes) -> StorageResult<()> {
        let path = path_to_string(path)?;
        self.block_op()?
            .write(path.as_str(), bs)
            .map_err(StorageError::from)
    }

    fn remove_file(&self, path: std::path::PathBuf) -> StorageResult<()> {
        let path = path_to_string(path)?;
        self.block_op()?
            .remove(vec![path])
            .map_err(StorageError::from)
    }

    async fn create_dir(&self, path: std::path::PathBuf) -> StorageResult<()> {
        let path = path_to_string(path)?;
        let path = if path.ends_with("/") {
            path.to_string()
        } else {
            format!("{}/", path)
        };
        self.op()?
            .create_dir(path.as_str())
            .await
            .map_err(StorageError::from)
    }

    async fn is_exist(&self, path: std::path::PathBuf) -> StorageResult<bool> {
        let path = path_to_string(path)?;
        self.op()?
            .is_exist(path.as_str())
            .await
            .map_err(StorageError::from)
    }

    async fn read(&self, path: std::path::PathBuf) -> StorageResult<Buffer> {
        let path = path_to_string(path)?;
        self.op()?
            .read(path.as_str())
            .await
            .map_err(StorageError::from)
    }

    async fn write(&self, path: std::path::PathBuf, bs: Buffer) -> StorageResult<()> {
        let path = path_to_string(path)?;
        self.op()?
            .write(path.as_str(), bs)
            .await
            .map_err(StorageError::from)
    }

    async fn copy(&self, from: std::path::PathBuf, to: std::path::PathBuf) -> StorageResult<()> {
        let to = path_to_string(to)?;
        // copy file between path(not under root of opendal) and opendal
        if !self.under_root(from.clone())? {
            let data = tokio::fs::read(from)
                .await
                .map_err(|e| StorageError::from(e))?;
            self.op()?
                .write(to.as_str(), data)
                .await
                .map_err(StorageError::from)
        } else {
            // copy file under root of opendal
            self.op()?
                .copy(path_to_string(from)?.as_str(), to.as_str())
                .await
                .map_err(StorageError::from)
        }
    }

    async fn read_dir(&self, path: std::path::PathBuf) -> StorageResult<Vec<std::path::PathBuf>> {
        let path = if path.ends_with("/") {
            path
        } else {
            path.join("/")
        };
        self.op()?
            .list(path_to_string(path)?.as_str())
            .await
            .map(|entries| {
                entries
                    .into_iter()
                    .map(|entry| PathBuf::from(entry.path()))
                    .collect::<Vec<PathBuf>>()
            })
            .map_err(StorageError::from)
    }

    async fn remove_dir_all(&self, path: std::path::PathBuf) -> StorageResult<()> {
        let path = path_to_string(path)?;
        self.op()?
            .remove_all(path.as_str())
            .await
            .map_err(StorageError::from)
    }

    async fn len(&self, path: std::path::PathBuf) -> StorageResult<u64> {
        let path = path_to_string(path)?;
        self.op()?
            .stat(path.as_str())
            .await
            .map(|stat| stat.content_length())
            .map_err(StorageError::from)
    }

    /// upload local dir recursively to opendal
    async fn upload_dir_recursive(
        &self,
        // relative path to root path
        dir: std::path::PathBuf,
    ) -> StorageResult<()> {
        // let actual_path = self.get_actual_path(dir)?;
        // for entry in std::fs::read_dir(actual_path)? {
        //     let entry_path = entry?.path();
        //     let path = entry_path
        //         .strip_prefix(self.root()?.as_os_str())
        //         .map_err(|_| StorageError::PathError)?
        //         .to_path_buf();

        //     if entry_path.is_dir() {
        //         self.upload_dir_recursive(path).await?;
        //     } else {
        //         let data = tokio::fs::read(&entry_path).await?;
        //         self.write(path, data.into()).await?;
        //     }
        // }
        Ok(())
    }

    async fn read_with_range(
        &self,
        path: std::path::PathBuf,
        range: std::ops::Range<u64>,
    ) -> StorageResult<Buffer> {
        let path = path_to_string(path)?;
        self.op()?
            .read_with(path.as_str())
            .range(range)
            .await
            .map_err(StorageError::from)
    }
}
