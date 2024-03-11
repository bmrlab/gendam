use prisma_lib::new_client_with_url;
use prisma_lib::PrismaClient;
use std::{path::PathBuf, sync::Arc};
use tokio::sync::RwLock;
use vector_db::{QdrantParams, QdrantServer};
// use tracing::info;

#[derive(Clone, Debug)]
pub struct Library {
    pub id: String,
    pub dir: PathBuf,
    pub files_dir: PathBuf, // for content files
    pub artifacts_dir: PathBuf,
    // db_url: String,
    pub prisma_client: Arc<RwLock<PrismaClient>>,
    pub qdrant_server: Arc<QdrantServer>,
}

pub async fn load_library(local_data_root: &PathBuf, library_id: &str) -> Library {
    let library_dir = local_data_root.join("libraries").join(library_id);
    let db_dir = library_dir.join("databases");
    let artifacts_dir = library_dir.join("artifacts");
    let files_dir = library_dir.join("files");
    let qdrant_dir = library_dir.join("qdrant");

    let db_url = format!(
        "file:{}?connection_limit=1",
        db_dir.join("muse-v2.db").to_str().unwrap()
    );
    let client = new_client_with_url(db_url.as_str())
        .await
        .expect("failed to create prisma client");
    client._db_push().await.expect("failed to push db"); // apply migrations
    let prisma_client = Arc::new(RwLock::new(client));

    let qdrant_server = QdrantServer::new(
        local_data_root.join("resources"),
        QdrantParams {
            dir: qdrant_dir,
            // TODO we should specify the port to avoid conflicts with other apps
            http_port: None,
            grpc_port: None,
        },
    )
    .await
    .expect("failed to start qdrant server");

    Library {
        id: library_id.to_string(),
        dir: library_dir,
        files_dir,
        artifacts_dir,
        // db_url,
        prisma_client,
        qdrant_server: Arc::new(qdrant_server),
    }
}

pub async fn create_library_with_title(local_data_root: &PathBuf, title: &str) -> Library {
    let _ = title;
    // TODO: 使用时间戳作为 id，当用户导入别人分享的 library 的时候,可能会冲突
    let library_id = sha256::digest(format!("{}", chrono::Utc::now()));
    let library_dir = local_data_root.join("libraries").join(&library_id);
    let db_dir = library_dir.join("databases");
    let qdrant_dir = library_dir.join("qdrant");
    let index_dir = library_dir.join("index");
    let artifacts_dir = library_dir.join("artifacts");
    let files_dir = library_dir.join("files");
    std::fs::create_dir_all(&db_dir).unwrap();
    std::fs::create_dir_all(&qdrant_dir).unwrap();
    std::fs::create_dir_all(&index_dir).unwrap();
    std::fs::create_dir_all(&artifacts_dir).unwrap();
    std::fs::create_dir_all(&files_dir).unwrap();
    load_library(local_data_root, &library_id).await
}

pub async fn upgrade_library_schemas(local_data_root: &PathBuf) {
    // TODO: 现在 load library 里面会进行 migrate, 这个方法可以不要了
    let _ = local_data_root;
    return;
    // let dirs = match local_data_root.join("libraries").read_dir() {
    //     Ok(dirs) => dirs,
    //     Err(e) => {
    //         info!("Failed to read libraries dir: {}", e);
    //         return;
    //     }
    // };
    // let dirs = dirs
    //     .into_iter()
    //     .filter(|entry| entry.as_ref().unwrap().path().is_dir())
    //     .map(|entry| entry.unwrap().path())
    //     .collect::<Vec<PathBuf>>();
    // for dir in dirs {
    //     let library_id = dir.file_name().unwrap().to_str().unwrap();
    //     let library = load_library(local_data_root, library_id).await;
    //     let client = new_client_with_url(library.db_url.as_str())
    //         .await
    //         .expect("failed to create prisma client");
    //     client._db_push().await.expect("failed to push db"); // apply migrations
    //     info!("Upgraded library '{}'", library_id);
    // }
}
