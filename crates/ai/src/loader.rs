use crate::Model;
use futures::Future;
use std::{pin::Pin, sync::Arc};
use tokio::sync::Mutex;
use tracing::debug;

pub(crate) struct ModelLoader<T>
where
    T: Model,
{
    pub model: Arc<Mutex<Option<T>>>,
    create_model_fn: Box<dyn Fn() -> Pin<Box<dyn Future<Output = anyhow::Result<T>>>> + Send>,
}

impl<T> ModelLoader<T>
where
    T: Model,
{
    pub fn new<F: Fn() -> Pin<Box<dyn Future<Output = anyhow::Result<T>>>> + Send + 'static>(
        create_model: F,
    ) -> Self {
        Self {
            model: Arc::new(Mutex::new(None)),
            create_model_fn: Box::new(create_model),
        }
    }

    pub async fn load(&self) -> anyhow::Result<()> {
        let mut current_model = self.model.lock().await;

        if current_model.is_none() {
            debug!("loading model");
            let model = (self.create_model_fn)().await?;
            *current_model = Some(model);
        }

        Ok(())
    }

    pub async fn offload(&self) -> anyhow::Result<()> {
        let mut current_model = self.model.lock().await;
        *current_model = None;

        Ok(())
    }
}
