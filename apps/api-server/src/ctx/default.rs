// Ctx 和 Store 的默认实现，主要给 api_server/main 用，不过目前 CtxWithLibrary 的实现也是可以给 tauri 用的，就先用着
use super::traits::{CtxStore, CtxWithLibrary, StoreError};
use crate::{
    ai::{models::get_model_info_by_id, AIHandler},
    download::{DownloadHub, DownloadReporter, DownloadStatus},
    library::get_library_settings,
    task_queue::{init_task_pool, trigger_unfinished, TaskPayload},
};
use async_trait::async_trait;
use content_library::{
    load_library, make_sure_collection_created, Library, QdrantCollectionInfo, QdrantServerInfo,
};
use file_handler::video::{VideoHandler, VideoTaskType};
use std::{
    boxed::Box,
    path::PathBuf,
    sync::{atomic::AtomicBool, mpsc::Sender, Arc, Mutex},
};
use vector_db::{get_language_collection_name, get_vision_collection_name, kill_qdrant_server};

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
    tx: Arc<Mutex<Option<Sender<TaskPayload<VideoHandler, VideoTaskType>>>>>,
    pub ai_handler: Arc<Mutex<Option<AIHandler>>>,
    library_loading: Arc<Mutex<AtomicBool>>,
    download_hub: Arc<Mutex<Option<DownloadHub>>>,
}

impl<S: CtxStore> Clone for Ctx<S> {
    fn clone(&self) -> Self {
        Self {
            local_data_root: self.local_data_root.clone(),
            resources_dir: self.resources_dir.clone(),
            store: self.store.clone(),
            current_library: self.current_library.clone(),
            tx: self.tx.clone(),
            ai_handler: self.ai_handler.clone(),
            library_loading: self.library_loading.clone(),
            download_hub: self.download_hub.clone(),
        }
    }
}

// pub const R: Rspc<Ctx> = Rspc::new();

impl<S: CtxStore> Ctx<S> {
    pub fn new(local_data_root: PathBuf, resources_dir: PathBuf, store: Arc<Mutex<S>>) -> Self {
        Self {
            local_data_root,
            resources_dir,
            store,
            current_library: Arc::new(Mutex::new(None)),
            tx: Arc::new(Mutex::new(None)),
            ai_handler: Arc::new(Mutex::new(None)),
            library_loading: Arc::new(Mutex::new(AtomicBool::new(false))),
            download_hub: Arc::new(Mutex::new(None)),
        }
    }
}

#[async_trait]
impl<S: CtxStore + Send> CtxWithLibrary for Ctx<S> {
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

    fn quit_library_in_store(&self) -> Result<(), rspc::Error> {
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

    async fn load_library(&self, library_id: &str) -> Result<(), rspc::Error> {
        tracing::info!("try to load library: {}", library_id);

        {
            let mut library_loading = self.library_loading.lock().unwrap();
            let is_loading = library_loading.get_mut();
            if *is_loading {
                // if library is already loading, just return
                // FIXME it's better to wait until library is loaded
                return Err(rspc::Error::new(
                    // FIXME should use 429 too many requests error code
                    rspc::ErrorCode::Conflict,
                    "too many requests".into(),
                ));
            }

            library_loading.store(true, std::sync::atomic::Ordering::Relaxed);
        }

        if let Err(e) = self.quit_library_in_store() {
            tracing::warn!("Failed to quit current library: {}", e);
        } else {
            tracing::info!("Quit current library successfully");
        }

        {
            let mut current_tx = self.tx.lock().unwrap();
            if current_tx.is_some() {
                if let Err(e) = current_tx.as_ref().unwrap().send(TaskPayload::CancelAll) {
                    tracing::warn!("Failed to send task cancel: {}", e);
                }
            }
            let tx = init_task_pool().expect("Failed to init task pool");
            *current_tx = Some(tx);
        }

        {
            let mut store = self.store.lock().unwrap();
            let _ = store.insert("current-library-id", library_id);
            if store.save().is_err() {
                tracing::warn!("Failed to save store");
            }
        }

        {
            // MutexGuard is not `Send`, so we have to write in this way
            // otherwise we can't send it to other thread, and compiler will throw error
            //
            // 下面是一种更常见的写法，但是会报错：原因是 MutexGuard 无法实现 Send
            // 这里由于 ai_handler 实现了 Clone，因此可以避免这个问题
            //
            //     👇 MutexGuard
            // let current_ai_handler = self.ai_handler.lock().unwrap();
            // if let Some(ai_handler) = &*current_ai_handler {
            //                                          👇 MutexGuard 还存在，但是调用了 await
            //     if let Err(e) = ai_handler.shutdown().await {
            //         tracing::warn!("Failed to shutdown AI handler: {}", e);
            //     }
            // }
            let current_ai_handler = {
                let current_ai_handler = self.ai_handler.lock().unwrap();
                let current_ai_handler = current_ai_handler.as_ref().map(|v| v.clone());
                current_ai_handler
            };
            if let Some(ai_handler) = current_ai_handler {
                if let Err(e) = ai_handler.shutdown().await {
                    tracing::warn!("Failed to shutdown AI handler: {}", e);
                }
            }
        }

        let library = load_library(&self.local_data_root, &library_id)
            .await
            .unwrap();

        let qdrant_client = library.qdrant_client();

        let pid = library.qdrant_server_info();
        self.current_library.lock().unwrap().replace(library);

        {
            let mut store = self.store.lock().unwrap();
            let _ = store.insert("current-qdrant-pid", &pid.to_string());
            if store.save().is_err() {
                tracing::warn!("Failed to save store");
            }
        }

        tracing::info!("Current library switched to {}", library_id);

        // init download hub
        {
            let download_hub = DownloadHub::new();
            self.download_hub.lock().unwrap().replace(download_hub);
        }

        // trigger model download according to new library
        {
            // TODO
        }

        // init AI handler after library is loaded
        {
            let ai_handler = AIHandler::new(self).map_err(|e| {
                rspc::Error::new(
                    rspc::ErrorCode::InternalServerError,
                    format!("Failed to init AI handler: {}", e),
                )
            })?;
            self.ai_handler.lock().unwrap().replace(ai_handler);
        }

        // make sure qdrant collections are created
        let qdrant_info = self.qdrant_info()?;
        if let Err(e) = make_sure_collection_created(
            qdrant_client.clone(),
            &qdrant_info.language_collection.name,
            qdrant_info.language_collection.dim as u64,
        )
        .await
        {
            tracing::warn!("Failed to check language collection: {}", e);
        }
        if let Err(e) = make_sure_collection_created(
            qdrant_client.clone(),
            &qdrant_info.vision_collection.name,
            qdrant_info.vision_collection.dim as u64,
        )
        .await
        {
            tracing::warn!("Failed to check vision collection: {}", e);
        }

        {
            let library_loading = self.library_loading.lock().unwrap();
            library_loading.store(false, std::sync::atomic::Ordering::Relaxed);
        }

        Ok(())

        // 这里本来应该触发一下未完成的任务
        // 但是不await的话，没有特别好的写法
        // 把这里的触发放到前端，前端切换完成后再触发一下接口
        // 这样用户操作也不会被 block
    }

    fn task_tx(&self) -> Result<Sender<TaskPayload<VideoHandler, VideoTaskType>>, rspc::Error> {
        match self.tx.lock().unwrap().as_ref() {
            Some(tx) => Ok(tx.clone()),
            None => Err(rspc::Error::new(
                rspc::ErrorCode::BadRequest,
                String::from("No task tx is set"),
            )),
        }
    }

    fn ai_handler(&self) -> Result<AIHandler, rspc::Error> {
        match self.ai_handler.lock().unwrap().as_ref() {
            Some(ai_handler) => Ok(ai_handler.clone()),
            None => Err(rspc::Error::new(
                rspc::ErrorCode::BadRequest,
                String::from("No ai handler is set"),
            )),
        }
    }

    fn ai_handler_mutex(&self) -> Arc<Mutex<Option<AIHandler>>> {
        self.ai_handler.clone()
    }

    fn download_reporter(&self) -> Result<DownloadReporter, rspc::Error> {
        match self.download_hub.lock().unwrap().as_ref() {
            Some(download_hub) => Ok(download_hub.get_reporter()),
            None => Err(rspc::Error::new(
                rspc::ErrorCode::BadRequest,
                String::from("No download reporter is set"),
            )),
        }
    }

    fn download_status(&self) -> Result<Vec<DownloadStatus>, rspc::Error> {
        match self.download_hub.lock().unwrap().as_ref() {
            Some(download_hub) => Ok(download_hub.get_file_list()),
            None => Err(rspc::Error::new(
                rspc::ErrorCode::BadRequest,
                String::from("No download status is set"),
            )),
        }
    }

    fn qdrant_info(&self) -> Result<QdrantServerInfo, rspc::Error> {
        let library = self.library()?;

        let settings = get_library_settings(&library.dir);
        let language_model = get_model_info_by_id(self, &settings.models.text_embedding)?;
        let vision_model = get_model_info_by_id(self, &settings.models.multi_modal_embedding)?;

        let language_dim = language_model.dim.ok_or(rspc::Error::new(
            rspc::ErrorCode::InternalServerError,
            String::from("Language model do not have dim"),
        ))?;
        let vision_dim = vision_model.dim.ok_or(rspc::Error::new(
            rspc::ErrorCode::InternalServerError,
            String::from("Vision model do not have dim"),
        ))?;

        Ok(QdrantServerInfo {
            language_collection: QdrantCollectionInfo {
                name: get_language_collection_name(&settings.models.text_embedding),
                dim: language_dim,
            },
            vision_collection: QdrantCollectionInfo {
                name: get_vision_collection_name(&settings.models.multi_modal_embedding),
                dim: vision_dim,
            },
        })
    }

    async fn trigger_unfinished_tasks(&self) -> () {
        if let Ok(library) = self.library() {
            // Box::pin(async move {

            // })
            if let Err(e) = trigger_unfinished(&library, self).await {
                tracing::warn!("Failed to trigger unfinished tasks: {}", e);
            }
        } else {
            // Box::pin(async move {})
        }
    }
}
