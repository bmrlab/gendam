use futures::Future;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::debug;

pub(crate) struct ModelLoader<T, TFn, TFut>
where
    T: Send,
    TFut: Future<Output = anyhow::Result<T>>,
    TFn: Fn() -> TFut,
{
    pub model: Arc<Mutex<Option<T>>>, // will be loaded lazily with `create_model_fn`
    create_model_fn: TFn,
}

impl<T, TFn, TFut> ModelLoader<T, TFn, TFut>
where
    T: Send,
    TFut: Future<Output = anyhow::Result<T>>,
    TFn: Fn() -> TFut,
{
    pub fn new(create_model: TFn) -> Self {
        Self {
            model: Arc::new(Mutex::new(None)),
            create_model_fn: create_model,
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
