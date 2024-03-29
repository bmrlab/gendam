// Ctx 和 Store 的默认实现，主要给 api_server/main 用，不过目前 CtxWithLibrary 的实现也是可以给 tauri 用的，就先用着
use super::traits::{CtxStore, CtxWithLibrary, StoreError};
use crate::{
    ai::{init_ai_handlers, AIHandler},
    task_queue::{init_task_pool, TaskPayload},
};
use content_library::{load_library, Library};
use std::{
    boxed::Box,
    path::PathBuf,
    pin::Pin,
    sync::{mpsc::Sender, Arc, Mutex},
};

/**
 * default impl of a store for rspc Ctx
 */

pub struct Store {
    path: PathBuf,
    values: std::collections::HashMap<String, String>,
}

impl Store {
    pub fn new(path: PathBuf) -> Self {
        let values = std::collections::HashMap::new();
        Self { values, path }
    }
}

impl CtxStore for Store {
    fn load(&mut self) -> Result<(), StoreError> {
        let file = std::fs::File::open(&self.path)
            .map_err(|e| StoreError(format!("Failed to open file: {}", e)))?;
        let reader = std::io::BufReader::new(file);
        let values: std::collections::HashMap<String, String> = serde_json::from_reader(reader)
            .map_err(|e| StoreError(format!("Failed to read file: {}", e)))?;
        self.values = values;
        Ok(())
    }
    fn save(&self) -> Result<(), StoreError> {
        let file = std::fs::File::create(&self.path)
            .map_err(|e| StoreError(format!("Failed to create file: {}", e)))?;
        serde_json::to_writer(file, &self.values)
            .map_err(|e| StoreError(format!("Failed to write file: {}", e)))?;
        Ok(())
    }
    fn insert(&mut self, key: &str, value: &str) -> Result<(), StoreError> {
        self.values.insert(key.to_string(), value.to_string());
        Ok(())
    }
    fn get(&self, key: &str) -> Option<String> {
        let value = self.values.get(key);
        match value {
            Some(value) => Some(value.to_owned()),
            None => None,
        }
    }
    fn delete(&mut self, key: &str) -> Result<(), StoreError> {
        self.values.remove(key);
        Ok(())
    }
}

/**
 * default impl of a rspc Ctx
 */

#[derive(Debug)]
pub struct Ctx<S: CtxStore> {
    local_data_root: PathBuf,
    resources_dir: PathBuf,
    store: Arc<Mutex<S>>,
    current_library: Arc<Mutex<Option<Library>>>,
    tx: Arc<Mutex<Sender<TaskPayload>>>,
    ai_handler: AIHandler,
}

impl<S: CtxStore> Clone for Ctx<S> {
    fn clone(&self) -> Self {
        Self {
            local_data_root: self.local_data_root.clone(),
            resources_dir: self.resources_dir.clone(),
            store: Arc::clone(&self.store),
            current_library: Arc::clone(&self.current_library),
            tx: Arc::clone(&self.tx),
            ai_handler: self.ai_handler.clone(),
        }
    }
}

// pub const R: Rspc<Ctx> = Rspc::new();

impl<S: CtxStore> Ctx<S> {
    pub fn new(
        local_data_root: PathBuf,
        resources_dir: PathBuf,
        store: Arc<Mutex<S>>,
        current_library: Arc<Mutex<Option<Library>>>,
    ) -> Self {
        let tx = init_task_pool().expect("Failed to init task pool");
        let tx = Arc::new(Mutex::new(tx));

        // FIXME need to handle error
        let ai_handler =
            init_ai_handlers(resources_dir.clone()).expect("Failed to init ai handlers");

        Self {
            local_data_root,
            resources_dir,
            store,
            current_library,
            tx,
            ai_handler,
        }
    }
}

impl<S: CtxStore> CtxWithLibrary for Ctx<S> {
    fn get_local_data_root(&self) -> PathBuf {
        self.local_data_root.clone()
    }

    fn get_resources_dir(&self) -> PathBuf {
        self.resources_dir.clone()
    }

    fn library(&self) -> Result<Library, rspc::Error> {
        match self.current_library.lock().unwrap().as_ref() {
            Some(library) => Ok(library.clone()),
            None => Err(rspc::Error::new(
                rspc::ErrorCode::BadRequest,
                String::from("No current library is set"),
            )),
        }
    }

    fn switch_current_library<'async_trait>(
        &'async_trait self,
        library_id: &'async_trait str,
    ) -> Pin<Box<dyn std::future::Future<Output = ()> + Send + 'async_trait>>
    where
        Self: Sync + 'async_trait,
    {
        // cancel all tasks
        let mut current_tx = self.tx.lock().unwrap();
        if let Err(e) = current_tx.send(TaskPayload::CancelAll) {
            tracing::warn!("Failed to send CancelAll task: {}", e);
        }
        let tx = init_task_pool().expect("Failed to init task pool");
        *current_tx = tx;

        let mut store = self.store.lock().unwrap();
        let _ = store.insert("current-library-id", library_id);
        let _ = store.save();
        // try to load library, but this is not necessary
        let _ = store.load();
        if let Some(library_id) = store.get("current-library-id") {
            let library_id = library_id.clone();
            return Box::pin(async move {
                let library = load_library(&self.local_data_root, &library_id)
                    .await
                    .unwrap();
                self.current_library.lock().unwrap().replace(library);
                tracing::info!("Current library switched to {}", library_id);
            });
        } else {
            // 这里实际上不可能被执行，除非 settings.json 数据有问题
            return Box::pin(async move {
                self.current_library.lock().unwrap().take();
                tracing::info!("Current library is unset");
            });
        }
    }

    fn get_task_tx(&self) -> Arc<Mutex<Sender<TaskPayload>>> {
        self.tx.clone()
    }

    fn get_ai_handler(&self) -> AIHandler {
        self.ai_handler.clone()
    }
}
