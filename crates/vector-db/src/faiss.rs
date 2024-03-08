use anyhow::bail;
use faiss::index::{IndexImpl, SearchResult};
use faiss::{IdMap, Index};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::mpsc::{self, Sender};
use tokio::sync::{oneshot, Mutex};
use tracing::{debug, warn};

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

struct EmbeddingIndex {
    pub index: IdMap<IndexImpl>,
    pub path: PathBuf,
}

#[derive(Clone, Debug)]
pub struct FaissIndex {
    // actually, IndexInfo can only be None when IndexPayload is Flush
    // we can find better way to
    index_tx: Arc<Sender<(IndexPayload, Option<IndexInfo>)>>,
}

impl FaissIndex {
    pub fn new() -> Self {
        let (tx, mut rx) = mpsc::channel::<(IndexPayload, Option<IndexInfo>)>(512);
        let index_tx = Arc::new(tx);

        let current: Option<EmbeddingIndex> = None;
        let current = Arc::new(Mutex::new(current));

        tokio::spawn(async move {
            loop {
                match rx.recv().await {
                    Some((payload, info)) => {
                        let current = current.clone();
                        let mut current = current.lock().await;

                        match (current.as_mut(), info) {
                            (Some(index), Some(info)) => {
                                if index.path != info.path {
                                    // flush current index to disk
                                    if let Err(e) =
                                        flush_index(&mut index.index, index.path.clone())
                                    {
                                        tracing::error!("flush index error: {}", e);
                                        continue;
                                    }

                                    // and load new index from disk
                                    match load_index(info.path.clone(), info.dim) {
                                        Ok(new_index) => {
                                            *current = Some(EmbeddingIndex {
                                                index: new_index,
                                                path: info.path.clone(),
                                            });
                                        }
                                        Err(e) => {
                                            tracing::error!("load index error: {:?}", e);
                                            continue;
                                        }
                                    }
                                }
                            }
                            (None, Some(info)) => {
                                // just load new index
                                match load_index(info.path.clone(), info.dim) {
                                    Ok(index) => {
                                        *current = Some(EmbeddingIndex {
                                            index,
                                            path: info.path.clone(),
                                        });
                                    }
                                    Err(e) => {
                                        tracing::error!("load index error: {:?}", e);
                                        continue;
                                    }
                                }
                            }
                            _ => {
                                // do nothing
                            }
                        }

                        let current = current.as_mut();

                        if current.is_none() {
                            tracing::error!("faiss current index is None");
                            continue;
                        }

                        let current = current.unwrap();

                        match payload {
                            IndexPayload::Data(payload) => {
                                let xids = faiss::Idx::new(payload.id as u64);

                                // try to remove data with id to avoid duplicate vector
                                if let Ok(ids_selector) =
                                    faiss::selector::IdSelector::batch(&[xids])
                                {
                                    // remove results can be safely ignored
                                    let _ = current.index.remove_ids(&ids_selector);
                                }

                                if let Err(e) = current
                                    .index
                                    .add_with_ids(payload.embedding.as_slice(), &[xids])
                                {
                                    tracing::error!("add index error: {}", e);
                                };
                            }
                            IndexPayload::Search(payload) => {
                                let results = current
                                    .index
                                    .search(payload.embedding.as_slice(), payload.limit);
                                if let Err(e) = payload.tx.send(
                                    results
                                        .map_err(|e| anyhow::anyhow!("search index error: {}", e)),
                                ) {
                                    tracing::error!("search index error: {:?}", e);
                                }
                            }
                            IndexPayload::Flush => {
                                if let Err(e) = flush_index(&mut current.index, &current.path) {
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

            // flush index at the end of the loop
            let mut current = current.lock().await;
            if let Some(current) = current.as_mut() {
                if let Err(e) = flush_index(&mut current.index, &current.path) {
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
            .send((IndexPayload::Flush, None))
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

fn load_index(path: impl AsRef<Path>, dim: Option<usize>) -> anyhow::Result<IdMap<IndexImpl>> {
    if path.as_ref().exists() {
        let index = faiss::read_index(path.as_ref().to_str().unwrap()).expect("");
        index
            .into_id_map()
            .map_err(|e| anyhow::anyhow!("load index error: {}", e))
    } else if let Some(dim) = dim {
        let index = faiss::index_factory(dim as u32, "Flat", faiss::MetricType::InnerProduct)
            .expect("failed to create index");

        faiss::IdMap::new(index).map_err(|e| anyhow::anyhow!("load index error: {}", e))
    } else {
        bail!(
            "index {} does not exist, and dim is not provided",
            path.as_ref().to_str().unwrap()
        );
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
        .add(1, vec![1.0, 2.0, 3.0], index_info.clone())
        .await
        .expect("failed to add");

    index.flush_current().await.expect("failed to flush");
}
