use super::{AIModelLoader, AIModelTx, AsImageEmbeddingModel, AsTextEmbeddingModel};
use crate::HandlerPayload;
use std::path::PathBuf;
use tokio::sync::mpsc;
use tokio::sync::oneshot;
use tracing::info;

#[derive(Clone)]
pub enum MultiModalEmbeddingInput {
    Image(PathBuf),
    Text(String),
}

pub type MultiModalEmbeddingOutput = Vec<f32>;

pub trait AsMultiModalEmbeddingModel: AsImageEmbeddingModel + AsTextEmbeddingModel {
    fn get_inputs_embedding_tx(
        &self,
    ) -> AIModelTx<MultiModalEmbeddingInput, MultiModalEmbeddingOutput>;
}

impl AsTextEmbeddingModel for AIModelLoader<MultiModalEmbeddingInput, MultiModalEmbeddingOutput> {
    fn get_texts_embedding_tx(&self) -> AIModelTx<String, MultiModalEmbeddingOutput> {
        let (tx, mut rx) = mpsc::channel::<
            HandlerPayload<(
                Vec<String>,
                oneshot::Sender<anyhow::Result<Vec<anyhow::Result<MultiModalEmbeddingOutput>>>>,
            )>,
        >(512);

        let self_tx = self.tx.clone();

        tokio::spawn(async move {
            loop {
                match rx.recv().await {
                    Some(HandlerPayload::BatchData(data)) => {
                        let results = self_tx
                            .process(
                                data.0
                                    .into_iter()
                                    .map(|v| MultiModalEmbeddingInput::Text(v))
                                    .collect(),
                            )
                            .await;

                        let _ = data.1.send(results);
                    }
                    Some(HandlerPayload::Shutdown) => {
                        info!("Shutdown texts_embedding_tx");
                        break;
                    }
                    _ => {
                        break;
                    }
                }
            }
        });

        AIModelTx { tx }
    }
}

impl AsImageEmbeddingModel for AIModelLoader<MultiModalEmbeddingInput, MultiModalEmbeddingOutput> {
    fn get_images_embedding_tx(&self) -> AIModelTx<PathBuf, MultiModalEmbeddingOutput> {
        let (tx, mut rx) = mpsc::channel::<
            HandlerPayload<(
                Vec<PathBuf>,
                oneshot::Sender<anyhow::Result<Vec<anyhow::Result<MultiModalEmbeddingOutput>>>>,
            )>,
        >(512);

        let self_tx = self.tx.clone();

        tokio::spawn(async move {
            loop {
                match rx.recv().await {
                    Some(HandlerPayload::BatchData(data)) => {
                        let results = self_tx
                            .process(
                                data.0
                                    .into_iter()
                                    .map(|v| MultiModalEmbeddingInput::Image(v))
                                    .collect(),
                            )
                            .await;

                        let _ = data.1.send(results);
                    }
                    Some(HandlerPayload::Shutdown) => {
                        info!("Shutdown images_embedding_tx");
                        break;
                    }
                    _ => break,
                }
            }
        });

        AIModelTx { tx }
    }
}

impl AsMultiModalEmbeddingModel
    for AIModelLoader<MultiModalEmbeddingInput, MultiModalEmbeddingOutput>
{
    fn get_inputs_embedding_tx(
        &self,
    ) -> AIModelTx<MultiModalEmbeddingInput, MultiModalEmbeddingOutput> {
        self.tx.clone()
    }
}
