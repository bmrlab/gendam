use content_base::db::DB;
use prisma_lib::PrismaClient;
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::sync::RwLock;

pub mod bundle;
mod database;
// mod port;

#[derive(Clone, Debug)]
pub struct Library {
    pub id: String,
    pub dir: PathBuf,
    files_dir: PathBuf, // for content files
    pub artifacts_dir: PathBuf,
    prisma_client: Arc<PrismaClient>,
    db: Arc<RwLock<DB>>,
}

impl Library {
    pub fn prisma_client(&self) -> Arc<PrismaClient> {
        Arc::clone(&self.prisma_client)
    }

    pub fn db(&self) -> Arc<RwLock<DB>> {
        Arc::clone(&self.db)
    }

    /// Get the absolute path of artifact directory on disk for a given file hash.
    ///
    /// Returns a dir like `%LIBRARY_DIR%/artifacts/%SHARD_ID%/%FILE_HASH%/`,
    /// where %SHARD_ID% is derived from the file hash.
    ///
    /// The artifacts directory will store all the artifacts for this file.
    ///
    /// DO NOT USE THIS FUNCTION, USE `relative_artifacts_dir` INSTEAD. Generally, files should be accessed through the OpenDAL interface.
    pub fn _absolute_artifacts_dir(&self, file_hash: &str) -> PathBuf {
        let artifacts_dir = self
            .artifacts_dir
            .join(get_shard_hex(file_hash))
            .join(file_hash);

        if !artifacts_dir.exists() {
            if let Err(e) = std::fs::create_dir_all(&artifacts_dir) {
                tracing::error!("Failed to create artifacts dir {:?}: {}", artifacts_dir, e);
            }
        }

        artifacts_dir
    }

    /// Get the relative path of artifact directory under library root for a given file hash.
    ///
    /// Returns a dir like `artifacts/%SHARD_ID%/%FILE_HASH%/`
    ///
    /// OpenDAL will create directory iteratively if not exist
    pub fn relative_artifacts_dir(&self, file_hash: &str) -> PathBuf {
        self.artifacts_dir_name()
            .join(get_shard_hex(file_hash))
            .join(file_hash)
    }

    pub fn artifacts_dir_name(&self) -> PathBuf {
        PathBuf::from("artifacts")
    }

    /// Get the absolute path of file directory on disk for a given file hash.
    ///
    /// Returns a dir like `%LIBRARY_DIR%/files/%SHARD_ID%/%FILE_HASH%/`
    ///
    /// DO NOT USE THIS FUNCTION, USE `relative_file_dir` INSTEAD
    fn _absolute_file_dir(&self, file_hash: &str) -> PathBuf {
        let file_dir_with_shard = self.files_dir.join(get_shard_hex(file_hash));
        if !file_dir_with_shard.exists() {
            std::fs::create_dir_all(&file_dir_with_shard).unwrap();
        }
        file_dir_with_shard.join(file_hash)
    }

    /// Get the relative path of file directory under library root for a given file hash.
    ///
    /// Returns a dir like `files/%SHARD_ID%/%FILE_HASH%/`,
    ///
    /// OpenDAL will create directory iteratively if not exist
    pub fn relative_file_dir(&self, file_hash: &str) -> PathBuf {
        self.files_dir_name()
            .join(get_shard_hex(file_hash))
            .join(file_hash)
    }

    pub fn files_dir_name(&self) -> PathBuf {
        PathBuf::from("files")
    }

    /// Get the absolute file path on disk for a given file hash
    ///
    /// Returns a file path like `%LIBRARY_DIR%/files/%SHARD_ID%/%FILE_HASH%/%VERBOSE_FILE_NAME%`
    ///
    /// Generally, files should be accessed through the OpenDAL interface.
    /// 注意！这个方法需要读取一次磁盘，所以不要频繁调用。目前主要是给文件处理服务用，没有问题。
    /// 一般来说，不需要读取本地文件，前端只需要预览图，只有查看原文件的时候才需要，这个也不频繁。
    pub fn file_full_path_on_disk(&self, file_hash: &str) -> PathBuf {
        let file_dir = self
            .files_dir
            .join(get_shard_hex(file_hash))
            .join(file_hash);
        let verbose_file_name = std::fs::read_to_string(file_dir.join("file.json"))
            .ok()
            .and_then(|content| serde_json::from_str::<serde_json::Value>(&content).ok())
            .and_then(|json| json["verbose_file_name"].as_str().map(|s| s.to_owned()))
            .unwrap_or("file_not_found".to_owned());
        file_dir.join(verbose_file_name)
    }
}

pub async fn load_library(
    local_data_root: impl AsRef<Path>,
    library_id: &str,
) -> Result<Library, ()> {
    let library_dir = local_data_root.as_ref().join("libraries").join(library_id);
    let db_dir = library_dir.join("databases");
    let artifacts_dir = library_dir.join("artifacts");
    let files_dir = library_dir.join("files");
    let surreal_dir = library_dir.join("surreal");

    let client = database::migrate_library(&db_dir).await?;

    let prisma_client = Arc::new(client);

    let dir = library_dir.to_str().ok_or(())?.to_string();

    global_variable::set_global_current_library!(library_id.to_string(), dir);

    let surrealdb_client = DB::new(surreal_dir).await.map_err(|e| {
        tracing::error!("Failed to create surrealdb client: {}", e);
    })?;
    let library = Library {
        id: library_id.to_string(),
        dir: library_dir.clone(),
        files_dir,
        artifacts_dir,
        prisma_client,
        db: Arc::new(RwLock::new(surrealdb_client)),
    };

    Ok(library)
}

pub async fn create_library(local_data_root: impl AsRef<Path>) -> PathBuf {
    let library_id = uuid::Uuid::new_v4().to_string();
    let library_dir = local_data_root.as_ref().join("libraries").join(&library_id);
    let db_dir = library_dir.join("databases");
    let artifacts_dir = library_dir.join("artifacts");
    let files_dir = library_dir.join("files");
    std::fs::create_dir_all(&db_dir).unwrap();
    std::fs::create_dir_all(&artifacts_dir).unwrap();
    std::fs::create_dir_all(&files_dir).unwrap();
    library_dir
}

pub fn list_library_dirs(local_data_root: impl AsRef<Path>) -> Vec<(String, String)> {
    let libraries_dir = local_data_root.as_ref().join("libraries");
    if !libraries_dir.exists() {
        return vec![];
    }
    let entries = match libraries_dir.read_dir() {
        Ok(entries) => entries,
        Err(e) => {
            tracing::error!("Failed to read libraries dir: {}", e);
            return vec![];
        }
    };
    let mut res: Vec<(String, String)> = vec![];
    for entry in entries {
        let library_dir = match entry.as_ref() {
            Ok(entry) => {
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }
                let path_str = match path.into_os_string().into_string() {
                    Ok(path_str) => path_str,
                    Err(_e) => continue,
                };
                let file_name_str = match entry.file_name().into_string() {
                    Ok(file_name_str) => file_name_str,
                    Err(_e) => continue,
                };
                (path_str, file_name_str)
            }
            Err(e) => {
                tracing::error!("Failed to read entry: {}", e);
                continue;
            }
        };
        res.push(library_dir);
    }
    res
}

fn get_shard_hex(hash: &str) -> &str {
    &hash[0..3]
}
