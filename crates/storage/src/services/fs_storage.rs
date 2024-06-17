use crate::{traits::Storage, utils::path_to_string, StorageError, StorageResult};
use async_trait::async_trait;
use opendal::{services::Fs, BlockingOperator, Operator};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct FsStorage {
    root: PathBuf,
    op: Operator,
    block_op: BlockingOperator,
}

impl FsStorage {
    pub fn new(root: impl AsRef<Path>) -> StorageResult<Self> {
        let root = path_to_string(root)?;
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
}

#[async_trait]
impl Storage for FsStorage {
    fn clone_box(&self) -> Box<dyn Storage> {
        Box::new(self.clone())
    }

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
        _: std::path::PathBuf,
    ) -> StorageResult<()> {
        Err(StorageError::NotImplemented)
    }
}

#[cfg(test)]
mod storage_test {
    use std::path::PathBuf;

    use opendal::Metakey;

    use crate::{add_tmp_suffix_to_path, services::s3_storage::S3Storage, S3Config, Storage};

    fn init_storage() -> super::FsStorage {
        let test_path = "/Users/zingerbee/Downloads/test/gendam";
        super::FsStorage::new(test_path).unwrap()
    }

    fn init_s3_storage() -> S3Storage {
        let test_path = "/Users/zingerbee/Downloads/test/gendam";
        S3Storage::new(
            test_path,
            S3Config::new(
                "my-test-bucket-131".into(),
                "http://127.0.0.1:9000".into(),
                "plEXyNod8DWttxmCt3Db".into(),
                "IuJYIdJIdJm8LWQgCXP7af9pmis0dz4soEs7vp0U".into(),
            ),
        )
        .unwrap()
    }

    fn clear_test_dir() {
        let test_path = "/Users/zingerbee/Downloads/test/gendam";
        let _ = std::fs::remove_dir_all(test_path);
    }

    #[test]
    fn test_under_root() {
        let storage = init_storage();
        assert!(!storage
            .under_root(PathBuf::from("/Users/zingerbee/Downloads/test.txt"))
            .unwrap());
        assert!(storage
            .under_root(PathBuf::from("/Users/zingerbee/Downloads/test/test.txt"))
            .unwrap());
    }

    #[tokio::test]
    async fn test_storage() {
        let storage = init_storage();
        let data = b"hello world".to_vec();
        storage
            .write(PathBuf::from("test.txt"), data.clone().into())
            .await
            .unwrap();
        let read_data = storage.read(PathBuf::from("test.txt")).await.unwrap();
        assert_eq!(read_data.to_vec(), data);
    }

    #[tokio::test]
    async fn test_copy() {
        let storage = init_storage();
        // copy file between path(not under root of opendal) and opendal
        let external_path = PathBuf::from("/Users/zingerbee/Downloads/test.txt");
        let external_mock_data = b"hello world external".to_vec();
        tokio::fs::write(external_path.clone(), external_mock_data.clone())
            .await
            .unwrap();

        storage
            .copy(external_path, PathBuf::from("test_external_copy.txt"))
            .await
            .unwrap();
        let external_data = storage
            .read(PathBuf::from("test_external_copy.txt"))
            .await
            .unwrap();
        assert_eq!(external_data.to_vec(), external_mock_data);

        // copy file under root of opendal
        let data = b"hello world".to_vec();
        storage
            .write(PathBuf::from("test.txt"), data.clone().into())
            .await
            .unwrap();
        storage
            .copy(PathBuf::from("test.txt"), PathBuf::from("test_copy.txt"))
            .await
            .unwrap();
        let read_data = storage.read(PathBuf::from("test_copy.txt")).await.unwrap();
        assert_eq!(read_data.to_vec(), data);
    }

    #[tokio::test]
    async fn test_write_file_but_dir_not_exist() {
        let storage = init_storage();
        let data = b"hello world".to_vec();
        let path = PathBuf::from("test_dir_not_exist/test_dir_not_exist2/test.txt");
        let result = storage.write(path, data.clone().into()).await;
        assert!(result.is_ok());
    }

    /// can not use absolute path in opendal
    #[tokio::test]
    async fn test_write_file_with_absolute_path() {
        let storage = init_storage();
        let data = b"hello world".to_vec();
        let path = PathBuf::from("/Users/zingerbee/Downloads/test/absolute_path_test.txt");
        let result = storage.write(path, data.clone().into()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_read_dir() {
        clear_test_dir();
        let storage = init_storage();

        let data = b"hello world".to_vec();
        storage
            .write(PathBuf::from("test.txt"), data.clone().into())
            .await
            .unwrap();
        storage
            .write(PathBuf::from("test2.txt"), data.clone().into())
            .await
            .unwrap();
        storage
            .write(PathBuf::from("fo/test.txt"), data.clone().into())
            .await
            .unwrap();
        storage
            .write(PathBuf::from("fo2/test.txt"), data.clone().into())
            .await
            .unwrap();

        {
            let read_res = storage.read_dir(PathBuf::from("fo/")).await.unwrap();
            assert_eq!(read_res.len(), 1);
            let read_res = storage.read_dir(PathBuf::from("fo")).await.unwrap();
            assert_eq!(read_res.len(), 1);
        }

        {
            let read_res = storage.read_dir(PathBuf::from("")).await.unwrap();
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
        storage
            .write(PathBuf::from("test.txt"), data.clone().into())
            .await
            .unwrap();
        storage
            .write(PathBuf::from("test2.txt"), data.clone().into())
            .await
            .unwrap();
        storage
            .write(PathBuf::from("fo/test.txt"), data.clone().into())
            .await
            .unwrap();
        assert!(storage.is_exist(PathBuf::from("test.txt")).await.unwrap());
        assert!(!storage.is_exist(PathBuf::from("test3.txt")).await.unwrap());
        assert!(storage
            .is_exist(PathBuf::from("fo/test.txt"))
            .await
            .unwrap());
        assert!(storage.is_exist(PathBuf::from("fo")).await.unwrap());
        assert!(storage.is_exist(PathBuf::from("fo/")).await.unwrap());
        clear_test_dir();
    }

    #[test]
    fn test_add_tmp_suffix_to_file() {
        let file_path = PathBuf::from("path/to/your/folder/aa.mp4");
        let new_path = add_tmp_suffix_to_path!(&file_path);
        println!("Original path: {:?}", file_path);
        println!("New path: {:?}", new_path);

        assert_eq!("path/to/your/folder/aa-tmp.mp4", new_path.to_str().unwrap());

        let file_path = PathBuf::from(
            "artifacts/bdc/bdca61586e79f6ba/audio-2762e699-07bb-4c81-b958-73325e0dedc5/transcript",
        );
        let new_path = add_tmp_suffix_to_path!(&file_path);
        println!("Original path: {:?}", file_path);
        println!("New path: {:?}", new_path);

        assert_eq!("artifacts/bdc/bdca61586e79f6ba/audio-2762e699-07bb-4c81-b958-73325e0dedc5/transcript-tmp", new_path.to_str().unwrap());
    }

    #[tokio::test]
    async fn test_metadata() {
        clear_test_dir();
        let storage = init_storage();
        let data = b"hello world".to_vec();
        storage
            .write(PathBuf::from("test.txt"), data.clone().into())
            .await
            .unwrap();
        let metadata = storage.op().unwrap().stat("test.txt").await.unwrap();
        println!("test: {:?}", metadata);

        storage
            .op()
            .unwrap()
            .write_with("test2.txt", data.clone().clone())
            .content_type("text/plain")
            .await
            .unwrap();
        let metadata = storage.op().unwrap().stat("test2.txt").await.unwrap();
        println!("test2: {:?}", metadata);

        let entry = storage
            .op()
            .unwrap()
            .list_with("")
            .metakey(
                Metakey::ContentLength
                    | Metakey::ContentType
                    | Metakey::ContentRange
                    | Metakey::LastModified,
            )
            .await
            .unwrap();
        println!("entry: {entry:?}");

        clear_test_dir();
    }

    #[tokio::test]
    async fn test_len() {
        let storage = init_storage();
        let data = b"hello world".to_vec();
        storage
            .write(PathBuf::from("test.txt"), data.clone().into())
            .await
            .unwrap();
        let len = storage.len(PathBuf::from("test.txt")).await.unwrap();
        assert_eq!(len, 11);
    }

    #[tokio::test]
    async fn test_upload_dir_recursive() {
        let storage = init_storage();
        let data = b"hello world".to_vec();
        storage
            .write(PathBuf::from("file/test.txt"), data.clone().into())
            .await
            .unwrap();
        storage
            .write(PathBuf::from("file/test2.txt"), data.clone().into())
            .await
            .unwrap();
        storage
            .write(PathBuf::from("file/fo/test.txt"), data.clone().into())
            .await
            .unwrap();
        storage
            .write(PathBuf::from("file/fo2/test.txt"), data.clone().into())
            .await
            .unwrap();

        let storage_s3 = init_s3_storage();

        storage_s3
            .upload_dir_recursive(PathBuf::from("file"))
            .await
            .unwrap();
    }
}
