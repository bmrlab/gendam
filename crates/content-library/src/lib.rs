use prisma_lib::new_client_with_url;
use prisma_lib::PrismaClient;
use qdrant_client::qdrant::OptimizersConfigDiff;
use std::{path::PathBuf, sync::Arc};
use vector_db::{QdrantParams, QdrantServer};

#[derive(Clone, Debug)]
pub struct Library {
    pub id: String,
    pub dir: PathBuf,
    pub files_dir: PathBuf, // for content files
    pub artifacts_dir: PathBuf,
    // db_url: String,
    prisma_client: Arc<PrismaClient>,
    pub qdrant_server: Arc<QdrantServer>,
}

impl Library {
    pub fn prisma_client(&self) -> Arc<PrismaClient> {
        Arc::clone(&self.prisma_client)
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
        "file:{}?connection_limit=1",
        db_dir.join("muse-v2.db").to_str().unwrap()
    );
    let client = new_client_with_url(db_url.as_str())
        .await
        .map_err(|_e| {
            tracing::error!("failed to create prisma client");
        })?;
    client
        ._db_push()
        .await // apply migrations
        .map_err(|e| {
            tracing::error!("failed to push db: {}", e);
        })?;
    let prisma_client = Arc::new(client);

    let qdrant_server = QdrantServer::new(QdrantParams {
        dir: qdrant_dir,
        // TODO we should specify the port to avoid conflicts with other apps
        http_port: None,
        grpc_port: None,
    })
    .await
    .map_err(|e| {
        tracing::error!("failed to start qdrant server: {}", e);
    })?;

    let qdrant = qdrant_server.get_client().clone();
    make_sure_collection_created(
        qdrant.clone(),
        vector_db::DEFAULT_COLLECTION_NAME,
        vector_db::DEFAULT_COLLECTION_DIM,
    )
    .await
    .map_err(|e| {
        tracing::error!(
            "failed to make sure collection created: {}, {}",
            vector_db::DEFAULT_COLLECTION_NAME,
            e
        );
    })?;

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
            let value = serde_json::json!({ title: title });
            if let Err(e) = serde_json::to_writer(file, &value) {
                tracing::error!("Failed to write file: {}", e);
            }
        },
        Err(e) => {
            tracing::error!("Failed to create file: {}", e);
        }
    };
    load_library(local_data_root, &library_id).await.unwrap()
}

use qdrant_client::client::QdrantClient;
use qdrant_client::qdrant::{
    vectors_config::Config, CreateCollection, Distance, VectorParams, VectorsConfig,
};
pub async fn make_sure_collection_created(
    qdrant: Arc<QdrantClient>,
    collection_name: &str,
    dim: u64,
) -> anyhow::Result<()> {
    async fn create(
        qdrant: Arc<QdrantClient>,
        collection_name: &str,
        dim: u64,
    ) -> anyhow::Result<()> {
        let res = qdrant
            .create_collection(&CreateCollection {
                collection_name: collection_name.to_string(),
                vectors_config: Some(VectorsConfig {
                    config: Some(Config::Params(VectorParams {
                        size: dim,
                        distance: Distance::Cosine.into(),
                        ..Default::default()
                    })),
                }),
                shard_number: Some(1),
                optimizers_config: Some(OptimizersConfigDiff {
                    default_segment_number: Some(1),
                    ..Default::default()
                }),
                ..Default::default()
            })
            .await;
        match res {
            Ok(_) => Ok(()),
            Err(e) => {
                tracing::error!("failed to create collection: {}, {:?}", collection_name, e);
                Err(e.into())
            }
        }
    }
    match qdrant.collection_info(collection_name).await {
        core::result::Result::Ok(info) => {
            if let None = info.result {
                create(qdrant, collection_name, dim).await
            } else {
                Ok(())
            }
        }
        Err(e) => {
            tracing::info!("collection info not found: {}, {:?}", collection_name, e);
            create(qdrant, collection_name, dim).await
        }
    }
}
