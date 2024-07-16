mod audio_transcript;
mod image_caption;
mod image_embedding;
mod llm;
mod multi_modal_embedding;
mod text_embedding;

use crate::{loader, HandlerPayload};
use anyhow::bail;
pub use audio_transcript::*;
use futures::Future;
pub use image_caption::*;
pub use image_embedding::*;
pub use llm::*;
pub use multi_modal_embedding::*;
use std::{sync::Arc, time::Duration};
pub use text_embedding::*;
use tokio::sync::{mpsc, oneshot, Mutex};
use tracing::{debug, error, warn};

pub trait Model {
    type Item;
    type Output;

    fn process(
        &mut self,
        items: Vec<Self::Item>,
    ) -> impl std::future::Future<Output = anyhow::Result<Vec<anyhow::Result<Self::Output>>>> + Send;
    fn batch_size_limit(&self) -> usize;
}

pub type BatchHandlerTx<Item, Output> = mpsc::Sender<HandlerPayload<Item, Output>>;

#[derive(Debug, Clone)]
pub struct AIModel<TItem, TOutput> {
    tx: BatchHandlerTx<TItem, TOutput>,
}

impl<TItem, TOutput> AIModel<TItem, TOutput>
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
        TFut: Future<Output = anyhow::Result<T>> + Send + 'static,
        TFn: Fn() -> TFut + Send + 'static,
    {
        let loader = loader::ModelLoader::new(create_model);
        let (tx, mut rx) = mpsc::channel::<HandlerPayload<TItem, TOutput>>(512);

        // TODO I think this is better: not offload model when offload_duration is None
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
                        payload = rx.recv() => {
                            match payload {
                                Some((items, result_tx)) => {
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
                                _ => {
                                    // this means all tx has been dropped
                                    if loader.model.lock().await.is_some() {
                                        warn!("all tx dropped, offload model and end loop");
                                        if let Err(e) = loader.offload().await {
                                            error!("failed to offload model: {}", e);
                                        }
                                    }
                                    break;
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

    pub async fn process(&self, items: Vec<TItem>) -> anyhow::Result<Vec<anyhow::Result<TOutput>>> {
        let (result_tx, rx) = oneshot::channel();

        self.tx.send((items, result_tx)).await?;
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

    pub fn create_reference<TNewItem, TNewOutput, TFnItem, TFnOutput>(
        &self,
        convert_item: TFnItem,
        convert_output: TFnOutput,
    ) -> AIModel<TNewItem, TNewOutput>
    where
        TNewItem: Send + Sync + Clone + 'static,
        TNewOutput: Send + Sync + Clone + 'static,
        TFnItem: Fn(TNewItem) -> TItem + Send + 'static,
        TFnOutput: Fn(TOutput) -> TNewOutput + Send + 'static,
    {
        let (tx, mut rx) = mpsc::channel::<HandlerPayload<TNewItem, TNewOutput>>(512);

        let self_clone = self.clone();

        tokio::spawn(async move {
            loop {
                match rx.recv().await {
                    Some(data) => {
                        let results = self_clone
                            .process(data.0.into_iter().map(|v| convert_item(v)).collect())
                            .await;

                        let results = results.map(|v| {
                            v.into_iter()
                                .map(|t| t.map(|k| convert_output(k)))
                                .collect()
                        });

                        let _ = data.1.send(results);
                    }
                    _ => {
                        break;
                    }
                }
            }
        });

        AIModel { tx }
    }
}

#[test_log::test(tokio::test)]
async fn test_create_reference() {
    use crate::clip::{CLIPModel, CLIP};

    let original_clip = AIModel::new(
        move || async move { CLIP::new("/Users/zhuo/dev/tezign/bmrlab/gendam/apps/desktop/src-tauri/resources/CLIP-ViT-B-32-multilingual-v1/visual_quantize.onnx", "/Users/zhuo/dev/tezign/bmrlab/gendam/apps/desktop/src-tauri/resources/CLIP-ViT-B-32-multilingual-v1/textual_quantize.onnx", "/Users/zhuo/dev/tezign/bmrlab/gendam/apps/desktop/src-tauri/resources/CLIP-ViT-B-32-multilingual-v1/tokenizer.json", CLIPModel::MViTB32).await },
        Some(Duration::from_secs(120)),
    ).expect("create CLIP model");

    let text_embedding_reference: TextEmbeddingModel = (&original_clip).into();
    let image_embedding_reference: ImageEmbeddingModel = (&original_clip).into();

    let text_embedding_reference_new = text_embedding_reference.create_reference(|v| v, |v| v);

    tracing::info!("[TEST] original_clip: {:?}", original_clip);
    tracing::info!(
        "[TEST] text_embedding_reference: {:?}",
        text_embedding_reference
    );
    tracing::info!(
        "[TEST] image_embedding_reference: {:?}",
        image_embedding_reference
    );

    tracing::info!("[TEST] shutdown original");
    drop(original_clip);
    tokio::time::sleep(Duration::from_secs(5)).await;

    // after shutdown original, reference should work like normal
    let result = text_embedding_reference
        .process(vec!["hello".to_string()])
        .await;
    tracing::info!("[TEST] result: {:?}", result);

    tracing::info!("[TEST] shutdown text embedding");
    drop(text_embedding_reference);
    tokio::time::sleep(Duration::from_secs(5)).await;

    tracing::info!("[TEST] shutdown image embedding");
    drop(image_embedding_reference);
    tokio::time::sleep(Duration::from_secs(5)).await;

    // after text embedding reference, new reference should work like normal
    let result = text_embedding_reference_new
        .process(vec!["hello".to_string()])
        .await;
    tracing::info!("[TEST] result: {:?}", result);

    tracing::info!("[TEST] shutdown text embedding new");
    drop(text_embedding_reference_new);

    tokio::time::sleep(Duration::from_secs(30)).await;
}
