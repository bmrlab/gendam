// Ctx 和 Store 的默认实现，主要给 api_server/main 用，不过目前 CtxWithLibrary 的实现也是可以给 tauri 用的，就先用着
use super::traits::{CtxStore, CtxWithLibrary, StoreError};
use crate::{
    ai::{init_ai_handlers, AIHandler},
    task_queue::{init_task_pool, trigger_unfinished, TaskPayload},
};
use content_library::{load_library, Library};
use file_handler::video::{VideoHandler, VideoTaskType};
use std::{
    boxed::Box,
    path::PathBuf,
    pin::Pin,
    sync::{atomic::AtomicBool, mpsc::Sender, Arc, Mutex},
};
use vector_db::kill_qdrant_server;

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
    tx: Arc<Mutex<Sender<TaskPayload<VideoHandler, VideoTaskType>>>>,
    ai_handler: AIHandler,
    library_loading: Arc<Mutex<AtomicBool>>,
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
            library_loading: self.library_loading.clone(),
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
            library_loading: Arc::new(Mutex::new(AtomicBool::new(false))),
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

    fn quit_library_in_store<'async_trait>(&self) -> Result<(), rspc::Error> {
        let mut store = self.store.lock().unwrap();
        let qdrant_pid = store.get("current-qdrant-pid");

        match qdrant_pid {
            Some(pid) => {
                if let Ok(pid) = pid.parse() {
                    kill_qdrant_server(pid).map_err(|e| {
                        rspc::Error::new(rspc::ErrorCode::InternalServerError, e.to_string())
                    })?;
                }
                let _ = store.delete("current-qdrant-pid");
            }
            _ => {}
        }

        if store.save().is_err() {
            tracing::warn!("Failed to save store");
        }

        Ok(())
    }

    fn library_id_in_store(&self) -> Option<String> {
        let store = self.store.lock().unwrap();
        store.get("current-library-id")
    }

    fn load_library<'async_trait>(
        &'async_trait self,
        library_id: &'async_trait str,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<(), rspc::Error>> + Send + 'async_trait>>
    where
        Self: Sync + 'async_trait,
    {
        tracing::info!("try to load library: {}", library_id);

        let mut library_loading = self.library_loading.lock().unwrap();
        let is_loading = library_loading.get_mut();
        if *is_loading {
            // if library is already loading, just return
            // FIXME it's better to wait until library is loaded
            return Box::pin(async {
                Err(rspc::Error::new(
                    // FIXME should use 429 too many requests error code
                    rspc::ErrorCode::Conflict,
                    "too many requests".into(),
                ))
            });
        }

        library_loading.store(true, std::sync::atomic::Ordering::Relaxed);

        if let Err(e) = self.quit_library_in_store() {
            tracing::warn!("Failed to quit current library: {}", e);
        }

        tracing::info!("finished quit");

        let mut current_tx = self.tx.lock().unwrap();
        if let Err(e) = current_tx.send(TaskPayload::CancelAll) {
            tracing::warn!("Failed to send task cancel: {}", e);
        }
        let tx = init_task_pool().expect("Failed to init task pool");
        *current_tx = tx;

        let mut store = self.store.lock().unwrap();
        let _ = store.insert("current-library-id", library_id);
        if store.save().is_err() {
            tracing::warn!("Failed to save store");
        }

        // try to load library, but this is not necessary
        return Box::pin(async move {
            tracing::info!("start async loading library");
            let library = load_library(&self.local_data_root, &library_id)
                .await
                .unwrap();

            let pid = library.qdrant_server_info();
            self.current_library.lock().unwrap().replace(library);

            let mut store = self.store.lock().unwrap();
            let _ = store.insert("current-qdrant-pid", &pid.to_string());
            if store.save().is_err() {
                tracing::warn!("Failed to save store");
            }

            tracing::info!("Current library switched to {}", library_id);

            let library_loading = self.library_loading.lock().unwrap();
            library_loading.store(false, std::sync::atomic::Ordering::Relaxed);

            Ok(())

            // 这里本来应该触发一下未完成的任务
            // 但是不await的话，没有特别好的写法
            // 把这里的触发放到前端，前端切换完成后再触发一下接口
            // 这样用户操作也不会被 block
        });
    }

    fn get_task_tx(&self) -> Arc<Mutex<Sender<TaskPayload<VideoHandler, VideoTaskType>>>> {
        self.tx.clone()
    }

    fn get_ai_handler(&self) -> AIHandler {
        self.ai_handler.clone()
    }

    fn trigger_unfinished_tasks<'async_trait>(
        &'async_trait self,
    ) -> Pin<Box<dyn std::future::Future<Output = ()> + Send + 'async_trait>>
    where
        Self: Sync + 'async_trait,
    {
        if let Ok(library) = self.library() {
            Box::pin(async move {
                if let Err(e) = trigger_unfinished(&library, self).await {
                    tracing::warn!("Failed to trigger unfinished tasks: {}", e);
                }
            })
        } else {
            Box::pin(async move {})
        }
    }
}
