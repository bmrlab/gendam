use faiss::Index;
use std::{path::Path, sync::Arc};
use tokio::sync::mpsc::{self, Sender};
use tracing::debug;

pub struct EmbeddingPayload {
    id: u64,
    embedding: Vec<f32>,
}

pub enum IndexPayload {
    Data(EmbeddingPayload),
    Flush,
}

#[derive(Clone, Debug)]
pub struct EmbeddingIndex {
    pub index_tx: Arc<Sender<IndexPayload>>,
}

#[derive(Clone, Debug)]
pub struct VideoIndex {
    pub frame_index: EmbeddingIndex,
    pub frame_caption_index: EmbeddingIndex,
    pub transcript_index: EmbeddingIndex,
}

pub const VIDEO_FRAME_INDEX_NAME: &str = "frame-index";
pub const VIDEO_FRAME_CAPTION_INDEX_NAME: &str = "frame-caption-index";
pub const VIDEO_TRANSCRIPT_INDEX_NAME: &str = "transcript-index";

impl EmbeddingIndex {
    pub fn new(dir: impl AsRef<Path>, name: &str, dim: usize) -> anyhow::Result<Self> {
        debug!("start creating {} index", name);

        let filename = dir.as_ref().to_path_buf().join(name);
        debug!("filename: {}", filename.to_str().unwrap());

        let index = {
            if filename.exists() {
                debug!("index {} exists, reading it", name);
                let index = faiss::read_index(filename.to_str().unwrap())?;
                index.into_id_map()
            } else {
                debug!("index {} does not exist, creating it", name);
                let index = faiss::index_factory(
                    dim as u32,
                    "Flat",
                    faiss::MetricType::InnerProduct,
                )?;

                faiss::IdMap::new(index)
            }
        };

        debug!("index {} initialized", name);

        let name = name.to_string();

        match index {
            Ok(mut index) => {
                let (tx, mut rx) = mpsc::channel::<IndexPayload>(512);

                let name_cloned = name.clone();
                let dir_cloned = dir.as_ref().to_path_buf();
                tokio::spawn(async move {
                    loop {
                        match rx.recv().await {
                            Some(payload) => match payload {
                                IndexPayload::Data(payload) => {
                                    let xids = faiss::Idx::new(payload.id as u64);

                                    // try to remove data with id to avoid duplicate vector
                                    if let Ok(ids_selector) =
                                        faiss::selector::IdSelector::batch(&[xids])
                                    {
                                        let _ = index.remove_ids(&ids_selector);
                                    }

                                    if let Err(e) =
                                        index.add_with_ids(payload.embedding.as_slice(), &[xids])
                                    {
                                        tracing::error!("add {} index error: {}", &name_cloned, e);
                                    };
                                }
                                IndexPayload::Flush => {
                                    debug!(
                                        "index({}) has {} vectors",
                                        &name_cloned,
                                        index.ntotal()
                                    );

                                    debug!("index({}) dimension: {}", &name_cloned, index.d());

                                    if let Err(e) = faiss::write_index(
                                        &index,
                                        dir_cloned.join(&name_cloned).to_str().unwrap(),
                                    ) {
                                        tracing::error!(
                                            "flush {} index error: {}",
                                            &name_cloned,
                                            e
                                        );
                                    } else {
                                        tracing::info!("flush {} index success", &name_cloned);
                                    };
                                }
                            },
                            _ => {}
                        }
                    }
                });

                Ok(Self {
                    index_tx: Arc::new(tx),
                })
            }
            _ => {
                tracing::error!("create {} index error", name);
                Err(anyhow::anyhow!("create {} index error", name))
            }
        }
    }

    pub async fn add(&self, id: u64, embedding: Vec<f32>) -> anyhow::Result<()> {
        self.index_tx
            .send(IndexPayload::Data(EmbeddingPayload { id, embedding }))
            .await?;
        Ok(())
    }

    pub async fn flush(&self) -> anyhow::Result<()> {
        debug!("try to flush index");
        self.index_tx.send(IndexPayload::Flush).await?;
        Ok(())
    }
}

impl VideoIndex {
    pub fn new(dir: impl AsRef<Path>, dim: usize) -> anyhow::Result<Self> {
        debug!("start creating video index");
        let frame_index = EmbeddingIndex::new(dir.as_ref(), VIDEO_FRAME_INDEX_NAME, dim)?;
        let frame_caption_index =
            EmbeddingIndex::new(dir.as_ref(), VIDEO_FRAME_CAPTION_INDEX_NAME, dim)?;
        let transcript_index = EmbeddingIndex::new(dir.as_ref(), VIDEO_TRANSCRIPT_INDEX_NAME, dim)?;

        Ok(Self {
            frame_index,
            frame_caption_index,
            transcript_index,
        })
    }

    pub async fn flush(&self) -> anyhow::Result<()> {
        self.frame_index.flush().await?;
        self.frame_caption_index.flush().await?;
        self.transcript_index.flush().await?;
        Ok(())
    }
}
