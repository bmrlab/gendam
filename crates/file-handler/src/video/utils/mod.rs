use std::sync::Arc;
use tokio::sync::RwLock;
use vector_db::{FaissIndex, IndexInfo};

pub(crate) mod caption;
pub(crate) mod clip;
pub(crate) mod frame;
pub(crate) mod transcript;

pub async fn save_text_embedding(
    text: &str,
    id: u64,
    clip: Arc<RwLock<ai::clip::CLIP>>,
    embedding_index: FaissIndex,
    index_info: IndexInfo,
) -> anyhow::Result<()> {
    let embedding = clip.read().await.get_text_embedding(text).await?;
    let embedding: Vec<f32> = embedding.iter().map(|&x| x).collect();

    embedding_index.add(id, embedding, index_info).await?;

    Ok(())
}
