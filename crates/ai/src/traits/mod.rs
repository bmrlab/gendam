mod audio_transcript;
mod image_caption;
mod image_embedding;
mod llm;
mod multi_modal_embedding;
mod text_embedding;

use crate::{loader, HandlerPayload};
pub use audio_transcript::*;
use futures::Future;
pub use image_caption::*;
pub use image_embedding::*;
pub use llm::*;
pub use multi_modal_embedding::*;
use std::fmt::Debug;
use std::{collections::HashMap, sync::Arc, time::Duration};
pub use text_embedding::*;
use tokio::sync::{mpsc, oneshot, Mutex};

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

#[derive(Debug)]
pub struct AIModel<TItem, TOutput> {
    model_id: String, // for better logging
    tx: BatchHandlerTx<TItem, TOutput>,
}

impl<TItem, TOutput> Clone for AIModel<TItem, TOutput> {
    fn clone(&self) -> Self {
        Self {
            model_id: self.model_id.clone(),
            tx: self.tx.clone(),
        }
    }
}

impl<TItem, TOutput> AIModel<TItem, TOutput>
where
    TItem: Send + Sync + Clone + Debug + 'static,
    TOutput: Send + Sync + Debug + 'static,
{
    pub fn new<T, TFut, TFn>(
        model_id: String, // for better logging
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
                                tracing::debug!("No message received for {:?}, offload model", offload_duration);
                                if let Err(e) = loader.offload().await {
                                    tracing::error!("failed to offload model: {}", e);
                                }
                            }
                        }
                        payload = rx.recv() => {
                            match payload {
                                Some((items, result_tx)) => {
                                    if let Err(e) = loader.load().await {
                                        tracing::error!("failed to load model: {}", e);
                                        // TODO here we need to use tx
                                        // if let Err(_) = result_tx.send(Err(anyhow::anyhow!(e))) {
                                        //     tracing::error!("failed to send result");
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
                                                tracing::error!("failed to send results");
                                            }
                                        } else {
                                            tracing::error!("no valid model");
                                            if result_tx.send(Err(anyhow::anyhow!("failed to load model"))).is_err() {
                                                tracing::error!("failed to send results");
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
                                        tracing::warn!("all tx dropped, offload model and end loop");
                                        if let Err(e) = loader.offload().await {
                                            tracing::error!("failed to offload model: {}", e);
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

        Ok(Self {
            model_id: model_id.to_string(),
            tx,
        })
    }

    #[tracing::instrument(name = "AIModel::process", err(Debug), skip_all, fields(model_id=%self.model_id))]
    pub async fn process(&self, items: Vec<TItem>) -> anyhow::Result<Vec<anyhow::Result<TOutput>>> {
        let (result_tx, rx) = oneshot::channel();
        match self.tx.send((items, result_tx)).await {
            Ok(_) => {
                tracing::info!("items sent to model");
            }
            Err(e) => {
                anyhow::bail!("failed to send items: {:?}", e);
            }
        }

        match rx.await {
            Ok(result) => result,
            Err(e) => {
                anyhow::bail!("failed to receive results: {:?}", e);
            }
        }
    }

    #[tracing::instrument(name = "AIModel::process_single", err(Debug), skip_all, fields(model_id=%self.model_id))]
    pub async fn process_single(&self, item: TItem) -> anyhow::Result<TOutput> {
        let results = self.process(vec![item]).await?;
        let result = results
            .into_iter()
            .next()
            .ok_or(anyhow::anyhow!("no result"))??;
        Ok(result)
    }

    pub fn create_reference<TNewItem, TNewOutput, TItemFut, TNewOutputFut, TFnItem, TFnOutput>(
        &self,
        convert_item: TFnItem,
        convert_output: TFnOutput,
    ) -> AIModel<TNewItem, TNewOutput>
    where
        TNewItem: Send + Sync + Clone + 'static,
        TNewOutput: Send + Sync + Clone + 'static,
        TItemFut: Future<Output = Result<TItem, anyhow::Error>> + Send,
        TFnItem: Send + 'static + Fn(TNewItem) -> TItemFut,
        TNewOutputFut: Future<Output = Result<TNewOutput, anyhow::Error>> + Send,
        TFnOutput: Send + 'static + Fn(TOutput) -> TNewOutputFut,
    {
        let (tx, mut rx) = mpsc::channel::<HandlerPayload<TNewItem, TNewOutput>>(512);

        let self_clone = Self {
            model_id: self.model_id.clone(),
            tx: self.tx.clone(),
        };

        tokio::spawn(async move {
            loop {
                match rx.recv().await {
                    Some(data) => {
                        let mut input_data = vec![];
                        for v in data.0.into_iter() {
                            input_data.push(convert_item(v).await);
                        }

                        let total_len = input_data.len();

                        let mut raw_idx_to_error = HashMap::new();
                        let mut valid_input_data = vec![];
                        let mut raw_idx_to_result_idx = HashMap::new();

                        input_data
                            .into_iter()
                            .enumerate()
                            .for_each(|(idx, v)| match v {
                                Ok(v) => {
                                    raw_idx_to_result_idx.insert(idx, valid_input_data.len());
                                    valid_input_data.push(v);
                                }
                                Err(e) => {
                                    raw_idx_to_error.insert(idx, e);
                                }
                            });

                        let valid_results = self_clone.process(valid_input_data).await;

                        let results = match valid_results {
                            Ok(mut valid_results) => {
                                let mut results = vec![];

                                for i in 0..total_len {
                                    if let Some(e) = raw_idx_to_error.remove(&i) {
                                        results.push(Err(e));
                                    } else if let Some(idx) = raw_idx_to_result_idx.remove(&i) {
                                        let item_result = valid_results.remove(idx);
                                        match item_result {
                                            Ok(v) => {
                                                results.push(convert_output(v).await);
                                            }
                                            Err(e) => {
                                                results.push(Err(e));
                                            }
                                        }
                                    } else {
                                        unreachable!()
                                    }
                                }

                                Ok(results)
                            }
                            Err(e) => Err(e),
                        };

                        let _ = data.1.send(results);
                    }
                    _ => {
                        break;
                    }
                }
            }
        });

        AIModel {
            model_id: self.model_id.clone(),
            tx,
        }
    }
}

#[test_log::test(tokio::test)]
async fn test_create_reference() {
    use crate::clip::{CLIPModel, CLIP};

    let original_clip = AIModel::new(
        "clip-multilingual-v1".into(),
        move || async move { CLIP::new("/Users/zhuo/dev/tezign/bmrlab/gendam/apps/desktop/src-tauri/resources/CLIP-ViT-B-32-multilingual-v1/visual_quantize.onnx", "/Users/zhuo/dev/tezign/bmrlab/gendam/apps/desktop/src-tauri/resources/CLIP-ViT-B-32-multilingual-v1/textual_quantize.onnx", "/Users/zhuo/dev/tezign/bmrlab/gendam/apps/desktop/src-tauri/resources/CLIP-ViT-B-32-multilingual-v1/tokenizer.json", CLIPModel::MViTB32).await },
        Some(Duration::from_secs(120)),
    ).expect("create CLIP model");

    let text_embedding_reference: TextEmbeddingModel = (&original_clip).into();
    let image_embedding_reference: ImageEmbeddingModel = (&original_clip).into();

    let text_embedding_reference_new =
        text_embedding_reference.create_reference(|v| async { Ok(v) }, |v| async { Ok(v) });

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
