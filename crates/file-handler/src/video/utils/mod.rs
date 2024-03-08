use crate::search_payload::SearchPayload;
use qdrant_client::qdrant::{
    vectors_config::Config, CreateCollection, Distance, VectorParams, VectorsConfig,
};
use qdrant_client::{client::QdrantClient, qdrant::PointStruct};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::RwLock;

pub(crate) mod caption;
pub(crate) mod clip;
pub(crate) mod frame;
pub(crate) mod transcript;

pub async fn save_text_embedding(
    text: &str,
    payload: SearchPayload,
    clip: Arc<RwLock<ai::clip::CLIP>>,
    qdrant: Arc<QdrantClient>,
    collection_name: &str,
) -> anyhow::Result<()> {
    let embedding = clip.read().await.get_text_embedding(text).await?;
    let embedding: Vec<f32> = embedding.iter().map(|&x| x).collect();

    let point = PointStruct::new(
        payload.uuid(),
        embedding,
        json!(payload)
            .try_into()
            .map_err(|_| anyhow::anyhow!("invalid payload"))?,
    );
    qdrant
        .upsert_points(collection_name, None, vec![point], None)
        .await?;

    Ok(())
}

pub async fn make_sure_collection_created(
    qdrant: Arc<QdrantClient>,
    collection_name: &str,
    dim: u64,
) -> anyhow::Result<()> {
    match qdrant.collection_info(collection_name).await {
        core::result::Result::Ok(info) => match info.result {
            Some(_) => {}
            None => {
                qdrant
                    .create_collection(&CreateCollection {
                        collection_name: collection_name.to_string(),
                        vectors_config: Some(VectorsConfig {
                            config: Some(Config::Params(VectorParams {
                                size: dim,
                                distance: Distance::Cosine.into(),
                                ..Default::default()
                            })),
                        }),
                        ..Default::default()
                    })
                    .await?;
            }
        },
        Err(_) => {
            qdrant
                .create_collection(&CreateCollection {
                    collection_name: collection_name.to_string(),
                    vectors_config: Some(VectorsConfig {
                        config: Some(Config::Params(VectorParams {
                            size: dim,
                            distance: Distance::Cosine.into(),
                            ..Default::default()
                        })),
                    }),
                    ..Default::default()
                })
                .await?;
        }
    };

    Ok(())
}
