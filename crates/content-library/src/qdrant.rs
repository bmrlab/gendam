use crate::port::get_available_port;
use qdrant_client::{
    client::QdrantClient,
    qdrant::{
        vectors_config::Config, CreateCollection, Distance, OptimizersConfigDiff, VectorParams,
        VectorsConfig,
    },
};
use std::{path::Path, sync::Arc};
use vector_db::{QdrantParams, QdrantServer};

#[derive(Debug, Clone)]
pub struct QdrantCollectionInfo {
    pub name: String,
    pub dim: u32,
}

#[derive(Debug, Clone)]
pub struct QdrantServerInfo {
    pub language_collection: QdrantCollectionInfo,
    pub vision_collection: QdrantCollectionInfo,
}

pub async fn create_qdrant_server(qdrant_dir: impl AsRef<Path>) -> Result<QdrantServer, ()> {
    let http_port = get_available_port(6333, 8000).ok_or(())?;
    let grpc_port = get_available_port(http_port + 1, 8000).ok_or(())?;

    let qdrant_server = QdrantServer::new(QdrantParams {
        dir: qdrant_dir.as_ref().to_path_buf(),
        http_port: Some(http_port),
        grpc_port: Some(grpc_port),
    })
    .await
    .map_err(|e| {
        tracing::error!("failed to start qdrant server: {}", e);
    })?;

    // let qdrant = qdrant_server.get_client().clone();

    // for (collection_name, collection_dim) in vec![
    //     vector_db::DEFAULT_VISION_COLLECTION_NAME,
    //     vector_db::DEFAULT_LANGUAGE_COLLECTION_NAME,
    // ]
    // .iter()
    // .zip(vec![
    //     vector_db::DEFAULT_VISION_COLLECTION_DIM,
    //     vector_db::DEFAULT_LANGUAGE_COLLECTION_DIM,
    // ]) {
    //     make_sure_collection_created(qdrant.clone(), &collection_name, collection_dim)
    //         .await
    //         .map_err(|e| {
    //             tracing::error!(
    //                 "failed to make sure collection created: {}, {}",
    //                 collection_name,
    //                 e
    //             );
    //         })?;
    // }

    Ok(qdrant_server)
}

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
