use prisma_lib::{new_client_with_url, PrismaClient};
use qdrant_client::client::QdrantClient;
use std::{path::PathBuf, sync::Arc};
use vector_db::QdrantServer;
mod qdrant;
use qdrant::create_qdrant_server;

#[derive(Clone, Debug)]
pub struct Library {
    pub id: String,
    pub dir: PathBuf,
    // TODO files_dir can be set to private, for now it is used
    // in `apps/api-server/src/routes/files.rs` for debug
    pub files_dir: PathBuf, // for content files
    artifacts_dir: PathBuf,
    // db_url: String,
    prisma_client: Arc<PrismaClient>,
    qdrant_server: Arc<QdrantServer>,
}

impl Library {
    pub fn prisma_client(&self) -> Arc<PrismaClient> {
        Arc::clone(&self.prisma_client)
    }

    pub fn qdrant_client(&self) -> Arc<QdrantClient> {
        self.qdrant_server.get_client().clone()
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
}

pub async fn load_library(local_data_root: &PathBuf, library_id: &str) -> Result<Library, ()> {
    let library_dir = local_data_root.join("libraries").join(library_id);
    let db_dir = library_dir.join("databases");
    let artifacts_dir = library_dir.join("artifacts");
    let files_dir = library_dir.join("files");
    let qdrant_dir = library_dir.join("qdrant");

    let db_url = format!(
        // "file:{}?socket_timeout=1&connection_limit=10",
        "file:{}?socket_timeout=15&connection_limit=1",
        db_dir.join("muse-v2.db").to_str().unwrap()
    );
    let client = new_client_with_url(db_url.as_str()).await.map_err(|_e| {
        tracing::error!("failed to create prisma client");
    })?;
    client
        ._db_push()
        .await // apply migrations
        .map_err(|e| {
            tracing::error!("failed to push db: {}", e);
        })?;
    let prisma_client = Arc::new(client);

    let qdrant_server = create_qdrant_server(qdrant_dir).await?;

    let library = Library {
        id: library_id.to_string(),
        dir: library_dir,
        files_dir,
        artifacts_dir,
        prisma_client,
        qdrant_server: Arc::new(qdrant_server),
    };

    Ok(library)
}

pub async fn create_library_with_title(local_data_root: &PathBuf, title: &str) -> Library {
    let library_id = uuid::Uuid::new_v4().to_string();
    let library_dir = local_data_root.join("libraries").join(&library_id);
    let db_dir = library_dir.join("databases");
    let qdrant_dir = library_dir.join("qdrant");
    let artifacts_dir = library_dir.join("artifacts");
    let files_dir = library_dir.join("files");
    std::fs::create_dir_all(&db_dir).unwrap();
    std::fs::create_dir_all(&qdrant_dir).unwrap();
    std::fs::create_dir_all(&artifacts_dir).unwrap();
    std::fs::create_dir_all(&files_dir).unwrap();
    match std::fs::File::create(library_dir.join("settings.json")) {
        Ok(file) => {
            let value = serde_json::json!({ "title": title });
            if let Err(e) = serde_json::to_writer(file, &value) {
                tracing::error!("Failed to write file: {}", e);
            }
        }
        Err(e) => {
            tracing::error!("Failed to create file: {}", e);
        }
    };
    load_library(local_data_root, &library_id).await.unwrap()
}

pub fn list_libraries(local_data_root: &PathBuf) -> Vec<serde_json::Value> {
    let libraries_dir = local_data_root.join("libraries");
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
    let mut res: Vec<serde_json::Value> = vec![];
    for entry in entries {
        let (library_dir, library_id) = match entry.as_ref() {
            Ok(entry) => {
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }
                let file_name = match entry.file_name().to_str() {
                    Some(file_name) => file_name.to_string(),
                    None => {
                        tracing::error!("Failed to convert file name to string");
                        continue;
                    }
                };
                (path, file_name)
            }
            Err(e) => {
                tracing::error!("Failed to read library dir: {}", e);
                continue;
            }
        };
        let settings = get_library_settings(&library_dir);
        res.push(serde_json::json!({
            "id": library_id,
            "dir": library_dir,
            "settings": settings,
        }));
    }
    res
}

pub fn get_library_settings(library_dir: &PathBuf) -> serde_json::Value {
    match std::fs::File::open(library_dir.join("settings.json")) {
        Ok(file) => {
            let reader = std::io::BufReader::new(file);
            match serde_json::from_reader(reader) {
                Ok(values) => values,
                Err(e) => {
                    tracing::error!("Failed to read file: {}", e);
                    serde_json::json!({ "title": "Untitled" })
                }
            }
        }
        Err(e) => {
            tracing::error!("Failed to open library's settings.json, {}", e);
            serde_json::json!({ "title": "Untitled" })
        }
    }
}

pub fn set_library_settings(library_dir: &PathBuf, settings: serde_json::Value) {
    // create or update to library_dir.join("settings.json")
    match std::fs::File::create(library_dir.join("settings.json")) {
        Ok(file) => {
            if let Err(e) = serde_json::to_writer(file, &settings) {
                tracing::error!("Failed to write file: {}", e);
            }
        }
        Err(e) => {
            tracing::error!("Failed to create file: {}", e);
        }
    };
}

fn get_shard_hex(hash: &str) -> &str {
    &hash[0..3]
}
