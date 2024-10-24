use content_base::db::DB;
use global_variable::set_current;
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

    /// Get the artifact directory for a given file hash.
    ///
    /// The artifacts directory will store all the artifacts for this file.
    /// For now, `artifacts_dir` is something like `%LIBRARY_ARTIFACTS_DIR%/%SHARD_ID%/%FILE_HASH%`,
    /// where %SHARD_ID% is derived from the file hash.
    pub fn artifacts_dir(&self, file_hash: &str) -> PathBuf {
        let artifacts_dir_with_shard = self
            .artifacts_dir
            .join(get_shard_hex(&file_hash))
            .join(file_hash);

        if !artifacts_dir_with_shard.exists() {
            std::fs::create_dir_all(&artifacts_dir_with_shard).unwrap();
        }

        artifacts_dir_with_shard
    }

    pub fn relative_artifacts_path(&self, file_hash: &str) -> PathBuf {
        self.relative_artifacts_dir()
            .join(get_shard_hex(file_hash))
            .join(file_hash)
    }

    pub fn relative_artifacts_dir(&self) -> PathBuf {
        PathBuf::from("artifacts")
    }

    /// Get the file path in library for a given file hash
    ///
    /// For now, `file_path` is something like `%LIBRARY_FILES_DIR%/%SHARD_ID%/%FILE_HASH%`,
    /// where %SHARD_ID% is derived from the file hash.
    pub fn file_path(&self, file_hash: &str) -> PathBuf {
        let files_dir_with_shard = self.files_dir.join(get_shard_hex(file_hash));

        if !files_dir_with_shard.exists() {
            std::fs::create_dir_all(&files_dir_with_shard).unwrap();
        }

        files_dir_with_shard.join(file_hash)
    }

    pub fn relative_file_dir(&self) -> PathBuf {
        PathBuf::from("files")
    }

    /// opendal will create directory iteratively if not exist
    pub fn relative_file_path(&self, file_hash: &str) -> PathBuf {
        self.relative_file_dir()
            .join(get_shard_hex(file_hash))
            .join(file_hash)
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

    set_current!(library_id.to_string(), dir);
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
