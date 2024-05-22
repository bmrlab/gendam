mod error;

use crate::error::StorageError;
use error::Result;
use opendal::services::Fs;
use opendal::{Buffer, Operator};
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct Storage {
    root: PathBuf,
    op: Operator,
}

impl Storage {
    pub fn new_fs(root: &str) -> Result<Self> {
        let mut builder = Fs::default();
        builder.root(root);
        let op: Operator = Operator::new(builder)?.finish();
        Ok(Self {
            op,
            root: PathBuf::from(root),
        })
    }

    pub async fn read(&self, path: &str) -> Result<Buffer> {
        self.op.read(path).await.map_err(|e| e.into())
    }

    /// if dir not exist, create it iteratively
    pub async fn write(&self, path: &str, bs: impl Into<Buffer>) -> Result<()> {
        self.op.write(path, bs).await.map_err(|e| e.into())
    }

    pub fn operator(&self) -> &Operator {
        &self.op
    }

    // check if path is under root of opendal
    fn under_root(&self, path: &str) -> bool {
        let path = PathBuf::from(path);

        if path.is_relative() {
            return true;
        }

        let mut root_components = self.root.components();
        let mut path_components = path.components();

        root_components.all(|path_component| path_components.next() == Some(path_component))
    }

    pub async fn copy(&self, from: &str, to: &str) -> Result<()> {
        // copy file between path(not under root of opendal) and opendal
        if !self.under_root(from) {
            let data = tokio::fs::read(from)
                .await
                .map_err(|e| StorageError::from(e))?;
            self.op.write(to, data).await.map_err(|e| e.into())
        } else {
            // copy file under root of opendal
            self.op.copy(from, to).await.map_err(|e| e.into())
        }
    }
}

#[cfg(test)]
mod storage_test {

    fn init_storage() -> super::Storage {
        super::Storage::new_fs("/Users/zingerbee/Downloads/test").unwrap()
    }

    #[test]
    fn test_under_root() {
        let storage = init_storage();
        assert!(!storage.under_root("/Users/zingerbee/Downloads/test.txt"));
        assert!(storage.under_root("/Users/zingerbee/Downloads/test/test.txt"));
    }

    #[tokio::test]
    async fn test_storage() {
        let storage = init_storage();
        let data = b"hello world".to_vec();
        storage.write("test.txt", data.clone()).await.unwrap();
        let read_data = storage.read("test.txt").await.unwrap();
        assert_eq!(read_data.to_vec(), data);
    }

    #[tokio::test]
    async fn test_copy() {
        let storage = init_storage();
        // copy file between path(not under root of opendal) and opendal
        let external_path = "/Users/zingerbee/Downloads/test.txt";
        let external_mock_data = b"hello world external".to_vec();
        tokio::fs::write(external_path, external_mock_data.clone())
            .await
            .unwrap();

        storage
            .copy(external_path, "test_external_copy.txt")
            .await
            .unwrap();
        let external_data = storage.read("test_external_copy.txt").await.unwrap();
        assert_eq!(external_data.to_vec(), external_mock_data);

        // copy file under root of opendal
        let data = b"hello world".to_vec();
        storage.write("test.txt", data.clone()).await.unwrap();
        storage.copy("test.txt", "test_copy.txt").await.unwrap();
        let read_data = storage.read("test_copy.txt").await.unwrap();
        assert_eq!(read_data.to_vec(), data);
    }

    #[tokio::test]
    async fn test_write_file_but_dir_not_exist() {
        let storage = init_storage();
        let data = b"hello world".to_vec();
        let path = "test_dir_not_exist/test_dir_not_exist2/test.txt";
        let result = storage.write(path, data.clone()).await;
        assert!(result.is_ok());
    }

    /// can not use absolute path in opendal
    #[tokio::test]
    async fn test_write_file_with_absolute_path() {
        let storage = init_storage();
        let data = b"hello world".to_vec();
        let path = "/Users/zingerbee/Downloads/test/absolute_path_test.txt";
        let result = storage.write(path, data.clone()).await;
        assert!(result.is_ok());
    }
}
