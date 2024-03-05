use faiss::index::{IndexImpl, SearchResult};
use faiss::{IdMap, Index};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::mpsc::{self, Sender};
use tokio::sync::{oneshot, Mutex};
use tracing::{debug, error, warn};

pub struct EmbeddingPayload {
    id: u64,
    embedding: Vec<f32>,
}

pub struct SearchPayload {
    embedding: Vec<f32>,
    limit: usize,
    tx: oneshot::Sender<anyhow::Result<SearchResult>>,
}

pub enum IndexPayload {
    Data(EmbeddingPayload),
    Search(SearchPayload),
    Flush,
}

#[derive(Clone)]
pub struct IndexInfo {
    pub path: PathBuf,
    pub dim: Option<usize>,
}

pub const VIDEO_FRAME_INDEX_NAME: &str = "frame-index";
pub const VIDEO_FRAME_CAPTION_INDEX_NAME: &str = "frame-caption-index";
pub const VIDEO_TRANSCRIPT_INDEX_NAME: &str = "transcript-index";

#[derive(Clone, Debug)]
pub struct FaissIndex {
    pub index_tx: Arc<Sender<(IndexPayload, Option<IndexInfo>)>>,
}

impl FaissIndex {
    pub fn new() -> Self {
        let (tx, mut rx) = mpsc::channel::<(IndexPayload, Option<IndexInfo>)>(512);
        let index_tx = Arc::new(tx);
        let current_index_path: Option<PathBuf> = None;
        let current_index_path = Arc::new(Mutex::new(current_index_path));
        let current_index: Option<faiss::IdMap<IndexImpl>> = None;
        let current_index = Arc::new(Mutex::new(current_index));

        tokio::spawn(async move {
            loop {
                match rx.recv().await {
                    Some((payload, info)) => {
                        let current_index_path = current_index_path.clone();
                        let current_index = current_index.clone();

                        let mut current_index_path = current_index_path.lock().await;
                        let mut current_index = current_index.lock().await;

                        // FIXME this is a little stupid
                        let info = info.unwrap_or(IndexInfo {
                            path: current_index_path
                                .as_ref()
                                .unwrap_or(&PathBuf::new())
                                .clone(),
                            dim: None,
                        });

                        if current_index_path.is_none()
                            || &info.path != current_index_path.as_ref().unwrap()
                        {
                            let index = current_index.as_mut();
                            match index {
                                Some(index) => {
                                    // flush current index onto disk
                                    if let Err(e) =
                                        flush_index(index, current_index_path.as_ref().unwrap())
                                    {
                                        tracing::error!("flush index error: {}", e);
                                        continue;
                                    }
                                }
                                _ => {}
                            }

                            // load index
                            let index = {
                                if info.path.exists() {
                                    let index =
                                        faiss::read_index(info.path.to_str().unwrap()).expect("");
                                    index.into_id_map()
                                } else if let Some(dim) = info.dim {
                                    let index = faiss::index_factory(
                                        dim as u32,
                                        "Flat",
                                        faiss::MetricType::InnerProduct,
                                    )
                                    .expect("failed to create index");

                                    faiss::IdMap::new(index)
                                } else {
                                    error!(
                                        "index {} does not exist, and dim is not provided",
                                        info.path.to_str().unwrap()
                                    );
                                    continue;
                                }
                            };

                            match index {
                                Ok(index) => {
                                    debug!("dim: {}, ntotal: {}", index.d(), index.ntotal());
                                    if let Some(dim) = info.dim {
                                        if index.d() != dim as u32 {
                                            error!(
                                                "index {} has different dimension to {}",
                                                info.path.to_str().unwrap(),
                                                dim
                                            );
                                            continue;
                                        }
                                    }

                                    *current_index_path = Some(info.path.clone());
                                    *current_index = Some(index);
                                }
                                Err(e) => {
                                    tracing::error!(
                                        "load {} index error: {}",
                                        info.path.to_str().unwrap(),
                                        e
                                    );
                                }
                            }
                        }

                        let current_index = current_index.as_mut().unwrap();

                        match payload {
                            IndexPayload::Data(payload) => {
                                let xids = faiss::Idx::new(payload.id as u64);

                                // try to remove data with id to avoid duplicate vector
                                if let Ok(ids_selector) =
                                    faiss::selector::IdSelector::batch(&[xids])
                                {
                                    let _ = current_index.remove_ids(&ids_selector);
                                }

                                if let Err(e) = current_index
                                    .add_with_ids(payload.embedding.as_slice(), &[xids])
                                {
                                    tracing::error!("add index error: {}", e);
                                };
                            }
                            IndexPayload::Search(payload) => {
                                let results = current_index
                                    .search(payload.embedding.as_slice(), payload.limit);
                                if let Err(e) = payload.tx.send(
                                    results
                                        .map_err(|e| anyhow::anyhow!("search index error: {}", e)),
                                ) {
                                    tracing::error!("search index error: {:?}", e);
                                }
                            }
                            IndexPayload::Flush => {
                                if let Err(e) =
                                    flush_index(current_index, current_index_path.as_ref().unwrap())
                                {
                                    tracing::error!("flush index error: {}", e);
                                }
                            }
                        }
                    }
                    _ => {
                        warn!("FaissIndex index_tx channel closed");
                        break;
                    }
                }
            }

            let mut current_index = current_index.lock().await;
            let current_index_path = current_index_path.lock().await;

            if let (Some(index), Some(path)) = (current_index.as_mut(), current_index_path.as_ref())
            {
                if let Err(e) = flush_index(index, path) {
                    tracing::error!("flush index error: {}", e);
                }
            }
        });

        Self { index_tx }
    }

    pub async fn search(
        &self,
        embedding: Vec<f32>,
        limit: usize,
        index_info: IndexInfo,
    ) -> anyhow::Result<SearchResult> {
        let (tx, rx) = oneshot::channel();

        self.index_tx
            .send((
                IndexPayload::Search(SearchPayload {
                    embedding,
                    limit,
                    tx,
                }),
                Some(index_info),
            ))
            .await?;

        match rx.await {
            Ok(result) => result,
            Err(_) => Err(anyhow::anyhow!("search index error")),
        }
    }

    pub async fn add(
        &self,
        id: u64,
        embedding: Vec<f32>,
        index_info: IndexInfo,
    ) -> anyhow::Result<()> {
        self.index_tx
            .send((
                IndexPayload::Data(EmbeddingPayload { id, embedding }),
                Some(index_info),
            ))
            .await
            .map_err(|e| anyhow::anyhow!("add index error: {}", e))
    }

    pub async fn flush_current(&self) -> anyhow::Result<()> {
        self.index_tx
            .send((
                IndexPayload::Flush,
                // TODO maybe just make index_info optional
                None,
            ))
            .await
            .map_err(|e| anyhow::anyhow!("flush index error: {}", e))
    }
}

fn flush_index(index: &mut IdMap<IndexImpl>, path: impl AsRef<Path>) -> anyhow::Result<()> {
    if let Err(e) = faiss::write_index(index, path.as_ref().to_str().unwrap()) {
        tracing::error!("flush index error: {}", e);
        Err(anyhow::anyhow!("flush index error: {}", e))
    } else {
        Ok(())
    }
}

#[test_log::test(tokio::test)]
async fn test_faiss_index() {
    let index = FaissIndex::new();
    let index_info = IndexInfo {
        path: PathBuf::from("/Users/zhuo/Downloads/test.faiss"),
        dim: Some(3),
    };

    index
        .index_tx
        .send((
            IndexPayload::Data(EmbeddingPayload {
                id: 1,
                embedding: vec![1.0, 2.0, 3.0],
            }),
            Some(index_info.clone()),
        ))
        .await
        .unwrap();

    index
        .index_tx
        .send((IndexPayload::Flush, None))
        .await
        .unwrap();
}
