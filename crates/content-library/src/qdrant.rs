use crate::port::get_available_port;
use qdrant_client::{
    qdrant::{
        CreateCollectionBuilder, Distance, OptimizersConfigDiffBuilder, ScalarQuantizationBuilder,
        VectorParamsBuilder,
    },
    Qdrant,
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

    Ok(qdrant_server)
}

pub async fn make_sure_collection_created(
    qdrant: Arc<Qdrant>,
    collection_name: &str,
    dim: u64,
) -> anyhow::Result<()> {
    async fn create(qdrant: Arc<Qdrant>, collection_name: &str, dim: u64) -> anyhow::Result<()> {
        let res = qdrant
            .create_collection(
                CreateCollectionBuilder::new(collection_name)
                    .vectors_config(
                        VectorParamsBuilder::new(dim, Distance::Cosine)
                            .quantization_config(ScalarQuantizationBuilder::default()),
                    )
                    .optimizers_config(
                        OptimizersConfigDiffBuilder::default().default_segment_number(1),
                    ),
            )
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
