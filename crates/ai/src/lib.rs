use anyhow::bail;
use async_trait::async_trait;
use derivative::Derivative;
use futures::Future;
use std::{sync::Arc, time::Duration};
use tokio::sync::{mpsc, oneshot, Mutex};
use tracing::{debug, error, info};

pub mod blip;
pub mod clip;
mod loader;
pub mod preprocess;
pub mod text_embedding;
pub mod utils;
pub mod whisper;
pub mod moondream;
pub mod yolo;

enum HandlerPayload<T> {
    BatchData(T),
    Shutdown,
}

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

type BatchHandlerTx<Item, Output> = mpsc::Sender<
    HandlerPayload<(
        Vec<Item>,
        oneshot::Sender<anyhow::Result<Vec<anyhow::Result<Output>>>>,
    )>,
>;

#[derive(Derivative)]
#[derivative(Clone, Debug)]
pub struct BatchHandler<T>
where
    T: Model + 'static,
{
    tx: BatchHandlerTx<T::Item, T::Output>,
}

impl<T> BatchHandler<T>
where
    T: Model + Send + Sync,
    T::Item: Send + Sync + Clone + 'static,
    T::Output: Send + Sync + 'static,
{
    pub fn new<TFut, TFn>(
        create_model: TFn,
        offload_duration: Option<Duration>,
    ) -> anyhow::Result<Self>
    where
        TFut: Future<Output = anyhow::Result<T>> + 'static,
        TFn: Fn() -> TFut + Send + 'static,
    {
        let loader = loader::ModelLoader::new(create_model);

        let (tx, mut rx) = mpsc::channel::<
            HandlerPayload<(
                Vec<T::Item>,
                oneshot::Sender<anyhow::Result<Vec<anyhow::Result<T::Output>>>>,
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
                                            let batch_limit = model.batch_size_limit();
                                            let mut results = vec![];

                                            for chunk in items.as_slice().chunks(batch_limit) {
                                                let chunk = chunk.to_vec();

                                                match model.process(chunk).await {
                                                    Ok(res) => {
                                                        results.extend(res);
                                                    }
                                                    Err(e) => {
                                                        error!("failed to process chunk: {}", e);
                                                        results.extend(vec![Err(anyhow::anyhow!(e))]);
                                                    }
                                                }
                                            }

                                            if result_tx.send(Ok(results)).is_err() {
                                                error!("failed to send results");
                                            }
                                        } else {
                                            error!("failed to load model");
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

        Ok(Self { tx })
    }

    pub async fn process(
        &self,
        items: Vec<T::Item>,
    ) -> anyhow::Result<Vec<anyhow::Result<T::Output>>> {
        let (tx, rx) = oneshot::channel();

        self.tx.send(HandlerPayload::BatchData((items, tx))).await?;
        match rx.await {
            Ok(result) => result,
            Err(e) => {
                bail!("failed to receive results: {}", e);
            }
        }
    }

    pub async fn process_single(&self, item: T::Item) -> anyhow::Result<T::Output> {
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

#[test_log::test(tokio::test)]
async fn test_batch_handler() {
    let resources_dir =
        "/Users/zhuo/dev/tezign/bmrlab/tauri-dam-test-playground/apps/desktop/src-tauri/resources";

    let handler = BatchHandler::new(
        move || {
            Box::pin(async move {
                crate::blip::BLIP::new(resources_dir, crate::blip::BLIPModel::Base).await
            })
        },
        Some(Duration::from_secs(5)),
    )
    .expect("failed to create handler");

    let result = handler
        .process(vec![
            std::path::PathBuf::from("/Users/zhuo/Pictures/IMG_4551 3.JPG"),
            std::path::PathBuf::from("/Users/zhuo/Pictures/avatar.JPG"),
        ])
        .await
        .expect("failed to process items");

    tracing::info!("result: {:?}", result);

    tokio::time::sleep(Duration::from_secs(3)).await;

    let result = handler
        .process(vec![std::path::PathBuf::from(
            "/Users/zhuo/Pictures/avatar.JPG",
        )])
        .await
        .expect("failed to process items");

    tracing::info!("result: {:?}", result);

    tokio::time::sleep(Duration::from_secs(8)).await;

    let result = handler
        .process(vec![std::path::PathBuf::from(
            "/Users/zhuo/Pictures/avatar.JPG",
        )])
        .await
        .expect("failed to process items");

    tracing::info!("result: {:?}", result);

    tokio::time::sleep(Duration::from_secs(30)).await;
}
