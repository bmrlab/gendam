mod audio_transcript;
mod image_caption;
mod image_embedding;
mod multi_modal_embedding;
mod text_embedding;

use crate::{loader, HandlerPayload};
use anyhow::bail;
use async_trait::async_trait;
pub use audio_transcript::*;
use futures::Future;
pub use image_caption::*;
pub use image_embedding::*;
pub use multi_modal_embedding::*;
use std::{sync::Arc, time::Duration};
pub use text_embedding::*;
use tokio::sync::{mpsc, oneshot, Mutex};
use tracing::{debug, error, info};

#[async_trait]
pub trait Model {
    type Item;
    type Output;

    async fn process(
        &mut self,
        items: Vec<Self::Item>,
    ) -> anyhow::Result<Vec<anyhow::Result<Self::Output>>>;
    fn batch_size_limit(&self) -> usize;
}

pub type BatchHandlerTx<Item, Output> = mpsc::Sender<
    HandlerPayload<(
        Vec<Item>,
        oneshot::Sender<anyhow::Result<Vec<anyhow::Result<Output>>>>,
    )>,
>;

#[derive(Debug, Clone)]
pub struct AIModelTx<TItem, TOutput> {
    pub tx: BatchHandlerTx<TItem, TOutput>,
}

impl<TItem, TOutput> AIModelTx<TItem, TOutput>
where
    TItem: Send + Sync + 'static,
    TOutput: Send + Sync + 'static,
{
    pub async fn process(&self, items: Vec<TItem>) -> anyhow::Result<Vec<anyhow::Result<TOutput>>> {
        let (result_tx, rx) = oneshot::channel();

        self.tx
            .send(HandlerPayload::BatchData((items, result_tx)))
            .await?;
        match rx.await {
            Ok(result) => result,
            Err(e) => {
                bail!("failed to receive results: {}", e);
            }
        }
    }

    pub async fn process_single(&self, item: TItem) -> anyhow::Result<TOutput> {
        let results = self.process(vec![item]).await?;
        let result = results
            .into_iter()
            .next()
            .ok_or(anyhow::anyhow!("no result"))??;
        Ok(result)
    }

    pub async fn shutdown(&self) -> anyhow::Result<()> {
        self.tx.send(HandlerPayload::Shutdown).await?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct AIModelLoader<TItem, TOutput> {
    pub tx: AIModelTx<TItem, TOutput>,
}

impl<TItem, TOutput> AIModelLoader<TItem, TOutput>
where
    TItem: Send + Sync + Clone + 'static,
    TOutput: Send + Sync + Clone + 'static,
{
    pub fn new<T, TFut, TFn>(
        create_model: TFn,
        offload_duration: Option<Duration>,
    ) -> anyhow::Result<Self>
    where
        T: Model<Item = TItem, Output = TOutput> + Send + 'static,
        TFut: Future<Output = anyhow::Result<T>> + 'static + Send,
        TFn: Fn() -> TFut + Send + 'static,
    {
        let loader = loader::ModelLoaderV2::new(create_model);

        let (tx, mut rx) = mpsc::channel::<
            HandlerPayload<(
                Vec<TItem>,
                oneshot::Sender<anyhow::Result<Vec<anyhow::Result<TOutput>>>>,
            )>,
        >(512);

        let cloned_tx = tx.clone();

        let offload_duration = offload_duration.unwrap_or(Duration::from_secs(5));

        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()?;

        std::thread::spawn(move || {
            let local = tokio::task::LocalSet::new();

            local.spawn_local(async move {
            let is_processing = Arc::new(Mutex::new(false));
            loop {
                tokio::select! {
                    _ = tokio::time::sleep(offload_duration) => {
                        if loader.model.lock().await.is_some() {
                            debug!("No message received for {:?}, offload model", offload_duration);
                            if let Err(e) = loader.offload().await {
                                error!("failed to offload model: {}", e);
                            }
                        }
                    }
                    _ = cloned_tx.closed() => {
                        info!("Channel closed, offload model");
                        if let Err(e) = loader.offload().await {
                            error!("failed to offload model: {}", e);
                        }
                        break;
                    }
                    payload = rx.recv() => {
                        match payload {
                            Some(HandlerPayload::BatchData((items, result_tx))) => {
                                if let Err(e) = loader.load().await {
                                    error!("failed to load model: {}", e);
                                    // TODO here we need to use tx
                                    // if let Err(_) = result_tx.send(Err(anyhow::anyhow!(e))) {
                                    //     error!("failed to send result");
                                    // }
                                }

                                // If channel closed,
                                // we have no way to response, just ignore task.
                                // This is very useful for task cancellation.
                                if !result_tx.is_closed() {
                                    {
                                        let mut is_processing = is_processing.lock().await;
                                        *is_processing = true;
                                    };

                                    let mut model = loader.model.lock().await;
                                    if let Some(model) = model.as_mut() {
                                        let results = model.process(items).await;

                                        if result_tx.send(results).is_err() {
                                            error!("failed to send results");
                                        }
                                    } else {
                                        error!("no valid model");
                                        if result_tx.send(Err(anyhow::anyhow!("failed to load model"))).is_err() {
                                            error!("failed to send results");
                                        }
                                    }

                                    {
                                        let mut is_processing = is_processing.lock().await;
                                        *is_processing = false;
                                    }
                                }
                            }
                            Some(HandlerPayload::Shutdown) => {
                                if let Err(e) = loader.offload().await {
                                    error!("failed to offload model in shutdown: {}", e);
                                }
                                break;
                            }
                            _ => {
                                error!("failed to receive payload");
                            }
                        }
                    }
                }
            }
        });

            rt.block_on(local);
        });

        Ok(Self {
            tx: AIModelTx { tx },
        })
    }
}

impl<TItem, TOutput> Into<AIModelLoader<TItem, TOutput>> for AIModelTx<TItem, TOutput> {
    fn into(self) -> AIModelLoader<TItem, TOutput> {
        AIModelLoader { tx: self }
    }
}
