mod error;
mod traits;

use crate::error::StorageError;
pub use bytes::Bytes;
pub use error::StorageResult;
use opendal::services::Fs;
pub use opendal::Buffer;
use opendal::{BlockingOperator, Operator};
use std::path::{Path, PathBuf};
use std::vec;
pub use traits::StorageTrait;

#[derive(Clone, Debug)]
pub struct Storage {
    root: PathBuf,
    op: Operator,
    block_op: BlockingOperator,
}

impl Storage {
    pub fn new_fs(root: impl AsRef<Path>) -> StorageResult<Self> {
        let root = Storage::path_to_string(root)?;
        let mut builder = Fs::default();
        builder.root(root.as_str());
        let op: Operator = Operator::new(builder)?.finish();

        let mut builder = Fs::default();
        builder.root(root.as_str());
        let block_op = Operator::new(builder)?.finish().blocking();

        Ok(Self {
            op,
            block_op,
            root: PathBuf::from(root),
        })
    }

    fn path_to_string(path: impl AsRef<Path>) -> StorageResult<String> {
        match path.as_ref().to_str() {
            Some(path) => Ok(path.to_string()),
            None => Err(StorageError::PathError),
        }
    }

    pub fn operator(&self) -> &Operator {
        &self.op
    }

    pub fn blocking_operator(&self) -> &BlockingOperator {
        &self.block_op
    }

    /// To indicate that a path is a directory, it is compulsory to include a trailing / in the path. Failure to do so may result in NotADirectory error being returned by OpenDAL.
    /// https://opendal.apache.org/docs/rust/opendal/struct.BlockingOperator.html#method.create_dir
    pub async fn create_dir(&self, path: PathBuf) -> StorageResult<()> {
        let path = Storage::path_to_string(path)?;
        let path = if path.ends_with("/") {
            path.to_string()
        } else {
            format!("{}/", path)
        };
        self.op
            .create_dir(path.as_str())
            .await
            .map_err(StorageError::from)
    }

    pub async fn is_exist(&self, path: impl AsRef<Path>) -> StorageResult<bool> {
        let path = Storage::path_to_string(path)?;
        self.op
            .is_exist(path.as_str())
            .await
            .map_err(StorageError::from)
    }

    pub async fn read(&self, path: impl AsRef<Path>) -> StorageResult<Buffer> {
        let path = Storage::path_to_string(path)?;
        self.op
            .read(path.as_str())
            .await
            .map_err(StorageError::from)
    }

    pub fn read_blocking(&self, path: impl AsRef<Path>) -> StorageResult<Buffer> {
        let path = Storage::path_to_string(path)?;
        self.block_op
            .read(path.as_str())
            .map_err(StorageError::from)
    }

    pub fn read_to_string(&self, path: impl AsRef<Path>) -> StorageResult<String> {
        let path = Storage::path_to_string(path)?;
        self.block_op
            .read(path.as_str())
            .map(|bs| String::from_utf8(bs.to_vec()).map_err(StorageError::from))?
            .map_err(StorageError::from)
    }

    /// if dir not exist, create it iteratively
    pub async fn write(&self, path: impl AsRef<Path>, bs: impl Into<Buffer>) -> StorageResult<()> {
        let path = Storage::path_to_string(path)?;
        self.op
            .write(path.as_str(), bs)
            .await
            .map_err(StorageError::from)
    }

    pub fn write_blocking(
        &self,
        path: impl AsRef<Path>,
        bs: impl Into<Bytes>,
    ) -> StorageResult<()> {
        let path = Storage::path_to_string(path)?;
        self.block_op
            .write(path.as_str(), bs)
            .map_err(StorageError::from)
    }

    // check if path is under root of opendal
    fn under_root(&self, path: impl AsRef<Path>) -> bool {
        path.as_ref().is_relative()
    }

    pub async fn copy(&self, from: impl AsRef<Path>, to: impl AsRef<Path>) -> StorageResult<()> {
        let from = Storage::path_to_string(from)?;
        let to = Storage::path_to_string(to)?;
        // copy file between path(not under root of opendal) and opendal
        if !self.under_root(from.clone()) {
            let data = tokio::fs::read(from)
                .await
                .map_err(|e| StorageError::from(e))?;
            self.op
                .write(to.as_str(), data)
                .await
                .map_err(StorageError::from)
        } else {
            // copy file under root of opendal
            self.op
                .copy(from.as_str(), to.as_str())
                .await
                .map_err(StorageError::from)
        }
    }

    pub fn get_actual_path(&self, path: impl AsRef<Path>) -> PathBuf {
        self.root.join(path)
    }

    // list all files under path
    // not recursion
    // accept relative path like "path/" "path" ""
    pub async fn read_dir(&self, path: impl AsRef<Path>) -> StorageResult<Vec<PathBuf>> {
        match path.as_ref().to_str() {
            Some(path) => {
                let path = if path.ends_with("/") {
                    path.to_string()
                } else {
                    format!("{}/", path)
                };
                self.op
                    .list(path.as_str())
                    .await
                    .map(|entries| {
                        entries
                            .into_iter()
                            .map(|entry| PathBuf::from(entry.path()))
                            .collect::<Vec<PathBuf>>()
                    })
                    .map_err(StorageError::from)
            }
            None => Err(StorageError::from(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Invalid path",
            ))),
        }
    }

    pub fn remove_file(&self, path: impl AsRef<Path>) -> StorageResult<()> {
        self.block_op
            .remove(vec![path
                .as_ref()
                .to_str()
                .ok_or_else(|| StorageError::PathError)?
                .to_string()])
            .map_err(StorageError::from)
    }

    pub async fn remove_dir_all(&self, path: impl AsRef<Path>) -> StorageResult<()> {
        let path = Storage::path_to_string(path)?;
        self.op
            .remove_all(path.as_str())
            .await
            .map_err(StorageError::from)
    }

    pub fn add_tmp_suffix_to_path(path: &Path) -> PathBuf {
        let mut new_path = path.to_path_buf();
        if let Some(file_stem) = new_path.file_stem() {
            let new_file_stem = format!("{}-tmp", file_stem.to_string_lossy());
            new_path.set_file_name(format!(
                "{}{}{}",
                new_file_stem,
                if let Some(_) = new_path.extension() {
                    "."
                } else {
                    ""
                },
                new_path.extension().unwrap_or_default().to_string_lossy()
            ));
        }
        new_path
    }
}

#[cfg(test)]
mod storage_test {
    use std::path::PathBuf;

    use crate::Storage;

    fn init_storage() -> super::Storage {
        let test_path = "/Users/zingerbee/Downloads/test/gendam";
        super::Storage::new_fs(test_path).unwrap()
    }

    fn clear_test_dir() {
        let test_path = "/Users/zingerbee/Downloads/test/gendam";
        let _ = std::fs::remove_dir_all(test_path);
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

    #[tokio::test]
    async fn test_read_dir() {
        clear_test_dir();
        let storage = init_storage();

        let data = b"hello world".to_vec();
        storage.write("test.txt", data.clone()).await.unwrap();
        storage.write("test2.txt", data.clone()).await.unwrap();
        storage.write("fo/test.txt", data.clone()).await.unwrap();
        storage.write("fo2/test.txt", data.clone()).await.unwrap();

        {
            let read_res = storage.read_dir("fo/").await.unwrap();
            assert_eq!(read_res.len(), 1);
            let read_res = storage.read_dir("fo").await.unwrap();
            assert_eq!(read_res.len(), 1);
        }

        {
            let read_res = storage.read_dir("").await.unwrap();
            assert_eq!(read_res.len(), 4);
        }
    }

    #[test]
    fn test_std_read_dir() {
        let path: Vec<std::path::PathBuf> =
            std::fs::read_dir("/Users/zingerbee/Downloads/test/gendam/fo/")
                .unwrap()
                .filter_map(|res| res.map(|e| e.path()).ok())
                .collect();

        println!("{:?}", path);
    }

    #[tokio::test]
    async fn test_is_exist() {
        clear_test_dir();
        let storage = init_storage();
        let data = b"hello world".to_vec();
        storage.write("test.txt", data.clone()).await.unwrap();
        storage.write("test2.txt", data.clone()).await.unwrap();
        storage.write("fo/test.txt", data.clone()).await.unwrap();
        assert!(storage.is_exist("test.txt").await.unwrap());
        assert!(!storage.is_exist("test3.txt").await.unwrap());
        assert!(storage.is_exist("fo/test.txt").await.unwrap());
        assert!(storage.is_exist("fo").await.unwrap());
        assert!(storage.is_exist("fo/").await.unwrap());
        clear_test_dir();
    }

    #[test]
    fn test_add_tmp_suffix_to_file() {
        let file_path = PathBuf::from("path/to/your/folder/aa.mp4");
        let new_path = Storage::add_tmp_suffix_to_path(&file_path);
        println!("Original path: {:?}", file_path);
        println!("New path: {:?}", new_path);

        assert_eq!("path/to/your/folder/aa-tmp.mp4", new_path.to_str().unwrap());

        let file_path = PathBuf::from(
            "artifacts/bdc/bdca61586e79f6ba/audio-2762e699-07bb-4c81-b958-73325e0dedc5/transcript",
        );
        let new_path = Storage::add_tmp_suffix_to_path(&file_path);
        println!("Original path: {:?}", file_path);
        println!("New path: {:?}", new_path);

        assert_eq!("artifacts/bdc/bdca61586e79f6ba/audio-2762e699-07bb-4c81-b958-73325e0dedc5/transcript-tmp", new_path.to_str().unwrap());
    }
}
