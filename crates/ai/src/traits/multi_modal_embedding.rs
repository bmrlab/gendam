use super::AIModel;
use crate::HandlerPayload;
use crate::ImageEmbeddingModel;
use crate::TextEmbeddingModel;
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
pub type MultiModalEmbeddingModel = AIModel<MultiModalEmbeddingInput, MultiModalEmbeddingOutput>;

impl Into<TextEmbeddingModel> for &MultiModalEmbeddingModel {
    fn into(self) -> TextEmbeddingModel {
        let (tx, mut rx) = mpsc::channel::<
            HandlerPayload<(
                Vec<String>,
                oneshot::Sender<anyhow::Result<Vec<anyhow::Result<MultiModalEmbeddingOutput>>>>,
            )>,
        >(512);

        let self_clone = self.clone();

        tokio::spawn(async move {
            loop {
                match rx.recv().await {
                    Some(HandlerPayload::BatchData(data)) => {
                        let results = self_clone
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
                        info!("Shutdown Into<TextEmbeddingModel> for MultiModalEmbeddingModel");
                        break;
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

impl Into<ImageEmbeddingModel> for &MultiModalEmbeddingModel {
    fn into(self) -> ImageEmbeddingModel {
        let (tx, mut rx) = mpsc::channel::<
            HandlerPayload<(
                Vec<PathBuf>,
                oneshot::Sender<anyhow::Result<Vec<anyhow::Result<MultiModalEmbeddingOutput>>>>,
            )>,
        >(512);

        let self_clone = self.clone();

        tokio::spawn(async move {
            loop {
                match rx.recv().await {
                    Some(HandlerPayload::BatchData(data)) => {
                        let results = self_clone
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
                        info!("Shutdown Into<ImageEmbeddingModel> for MultiModalEmbeddingModel");
                        break;
                    }
                    _ => break,
                }
            }
        });

        AIModel { tx }
    }
}
