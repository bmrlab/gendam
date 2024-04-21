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
    fmt::Debug,
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
    is_busy: Arc<Mutex<AtomicBool>>, // is loading or unloading a library
    local_data_root: PathBuf,
    resources_dir: PathBuf,
    store: Arc<Mutex<S>>,
    current_library: Arc<Mutex<Option<Library>>>,
    tx: Arc<Mutex<Option<Sender<TaskPayload<VideoHandler, VideoTaskType>>>>>,
    ai_handler: Arc<Mutex<Option<AIHandler>>>,
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
            is_busy: self.is_busy.clone(),
            download_hub: self.download_hub.clone(),
        }
    }
}

/*
 * is_busy 需要被实现一个 Drop trait，用于在离开作用域时自动释放锁
 * 不然当 load_library 和 unload_library 出错时，is_busy 会一直处于 true
 */
struct BusyGuard {
    pub is_busy: std::sync::Arc<std::sync::Mutex<std::sync::atomic::AtomicBool>>
}

impl Drop for BusyGuard {
    fn drop(&mut self) {
        tracing::info!("BusyGuard drop");
        let is_busy = self.is_busy.lock().unwrap();
        is_busy.store(false, std::sync::atomic::Ordering::Relaxed);
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
            is_busy: Arc::new(Mutex::new(AtomicBool::new(false))),
            download_hub: Arc::new(Mutex::new(None)),
        }
    }
}

fn unexpected_err(e: impl Debug) -> rspc::Error {
    rspc::Error::new(
        rspc::ErrorCode::InternalServerError,
        format!("unlock failed: {:?}", e),
    )
}

impl<S: CtxStore + Send> Ctx<S> {
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

#[async_trait]
impl<S: CtxStore + Send> CtxWithLibrary for Ctx<S> {
    fn is_busy(&self) -> Arc<Mutex<AtomicBool>> {
        self.is_busy.clone()
    }

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

    fn library_id_in_store(&self) -> Option<String> {
        let store = self.store.lock().unwrap();
        store.get("current-library-id")
    }

    #[tracing::instrument(level = "info", skip_all)] // create a span for better tracking
    async fn unload_library(&self) -> Result<(), rspc::Error> {
        {
            let mut is_busy = self.is_busy.lock().unwrap();
            if *is_busy.get_mut() {
                // FIXME should use 429 too many requests error code
                return Err(rspc::Error::new(
                    rspc::ErrorCode::Conflict,
                    "App is busy".into(),
                ));
            }
            is_busy.store(true, std::sync::atomic::Ordering::Relaxed);
        }
        // guard 需要放在后面, 这样前面返回 app is busy 的时候就不会触发 guard 的 drop
        let _guard = BusyGuard { is_busy: self.is_busy.clone() };

        /* cancel tasks */
        {
            let mut current_tx = self.tx.lock().map_err(unexpected_err)?;
            if let Some(tx) = current_tx.as_ref() {
                tx.send(TaskPayload::CancelAll).map_err(|e| {
                    tracing::error!(task = "cancel tasks", "Failed: {}", e);
                    rspc::Error::new(
                        rspc::ErrorCode::InternalServerError,
                        format!("Failed to cancel tasks: {}", e),
                    )
                })?;
                tracing::info!(task = "cancel tasks", "Success");
            }
            *current_tx = None; // same as current_tx.take();
        }

        /* kill qdrant */
        {
            let store = self.store.lock().map_err(unexpected_err)?;
            let pid_in_store = store.get("current-qdrant-pid").unwrap_or("".to_string());
            if let Ok(pid) = pid_in_store.parse() {
                kill_qdrant_server(pid).map_err(|e| {
                    tracing::error!(task = "kill qdrant", "Failed: {}", e);
                    rspc::Error::new(
                        rspc::ErrorCode::InternalServerError,
                        format!("Failed to kill qdrant: {}", e),
                    )
                })?;
                tracing::info!(task = "kill qdrant", "Success");
            }
        }

        /* shutdown ai handler */
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
                let ai_handler = self.ai_handler.lock().map_err(unexpected_err)?;
                let ai_handler = ai_handler.as_ref().map(|v| v.clone());
                ai_handler
            };
            if let Some(ai_handler) = current_ai_handler {
                ai_handler.shutdown().await.map_err(|e| {
                    tracing::error!(task = "shutdown ai handler", "Failed: {}", e);
                    rspc::Error::new(
                        rspc::ErrorCode::InternalServerError,
                        format!("Failed to shutdown AI handler: {}", e),
                    )
                })?;
                tracing::info!(task = "shutdown ai handler", "Success");
            }
        }

        /* update ctx */
        {
            let mut current_library = self.current_library.lock().map_err(unexpected_err)?;
            *current_library = None; // same as self.current_library.lock().unwrap().take();
            tracing::info!(task = "update ctx", "Success");
        }

        /* update store */
        {
            // TODO: 其实这里可以不删掉 library-id 和 qdrant-id 的，因为下次 load_library 的时候会自动覆盖
            let mut store = self.store.lock().map_err(unexpected_err)?;
            let _ = store.delete("current-library-id");
            let _ = store.delete("current-qdrant-pid");
            if let Err(e) = store.save() {
                tracing::warn!(task = "update store", "Failed: {:?}", e);
                // this issue can be safely ignored
            } else {
                tracing::info!(task = "update store", "Success");
            }
        }

        Ok(())
    }

    #[tracing::instrument(level = "info", skip_all)] // create a span for better tracking
    async fn load_library(&self, library_id: &str) -> Result<Library, rspc::Error> {
        {
            let mut is_busy = self.is_busy.lock().unwrap();
            if *is_busy.get_mut() {
                // FIXME should use 429 too many requests error code
                return Err(rspc::Error::new(
                    rspc::ErrorCode::Conflict,
                    "App is busy".into(),
                ));
            }
            is_busy.store(true, std::sync::atomic::Ordering::Relaxed);
        }
        // guard 需要放在后面, 这样前面返回 app is busy 的时候就不会触发 guard 的 drop
        let _guard = BusyGuard { is_busy: self.is_busy.clone() };

        if let Some(library) = self.current_library.lock().unwrap().as_ref() {
            if library.id == library_id {
                return Ok(library.clone());
            } else {
                return Err(rspc::Error::new(
                    rspc::ErrorCode::BadRequest,
                    format!(
                        "Library with diffrerent id {} is already loaded",
                        library.id
                    ),
                ));
            }
        }

        /* init library */
        let library = {
            let library = load_library(&self.local_data_root, &library_id)
                .await
                .map_err(|e| {
                    tracing::error!(task = "init library", "Failed: {:?}", e);
                    rspc::Error::new(
                        rspc::ErrorCode::InternalServerError,
                        format!("Failed to init library: {:?}", e),
                    )
                })?;
            tracing::info!(task = "init library", "Success");
            library
        };

        /* update ctx */
        {
            // should update ctx before self.qdrant_info() is called
            let mut current_library = self.current_library.lock().map_err(unexpected_err)?;
            current_library.replace(library.clone());
            tracing::info!(task = "update ctx", "Success");
        }

        /* update store */
        {
            let mut store = self.store.lock().map_err(unexpected_err)?;
            let _ = store.insert("current-library-id", library_id);
            let pid = library.qdrant_server_info();
            let _ = store.insert("current-qdrant-pid", &pid.to_string());
            if let Err(e) = store.save() {
                tracing::warn!(task = "update store", "Failed: {:?}", e);
                // this issue can be safely ignored
            } else {
                tracing::info!(task = "update store", "Success");
            }
        }

        /* check qdrant */
        {
            let qdrant_client = library.qdrant_client();
            // make sure qdrant collections are created
            let qdrant_info = self.qdrant_info()?;
            make_sure_collection_created(
                qdrant_client.clone(),
                &qdrant_info.language_collection.name,
                qdrant_info.language_collection.dim as u64,
            )
            .await
            .map_err(|e| {
                tracing::error!(task = "check qdrant", "Language collection error: {}", e);
                rspc::Error::new(
                    rspc::ErrorCode::InternalServerError,
                    format!("Language collection error: {}", e),
                )
            })?;
            make_sure_collection_created(
                qdrant_client.clone(),
                &qdrant_info.vision_collection.name,
                qdrant_info.vision_collection.dim as u64,
            )
            .await
            .map_err(|e| {
                tracing::error!(task = "check qdrant", "Vision collection error: {}", e);
                rspc::Error::new(
                    rspc::ErrorCode::InternalServerError,
                    format!("Vision collection error: {}", e),
                )
            })?;
            tracing::info!(
                task = "check qdrant",
                language_collection = qdrant_info.language_collection.name,
                vision_collection = qdrant_info.vision_collection.name,
                "Success"
            );
        }

        /* init task pool */
        {
            let mut current_tx = self.tx.lock().map_err(unexpected_err)?;
            let tx = init_task_pool().map_err(|e| {
                tracing::error!(task = "init task pool", "Failed: {}", e);
                rspc::Error::new(
                    rspc::ErrorCode::InternalServerError,
                    format!("Failed to init task pool: {}", e),
                )
            })?;
            tracing::info!(task = "init task pool", "Success");
            current_tx.replace(tx);
        }

        // init download hub
        {
            let download_hub = DownloadHub::new();
            let mut current_download_hub = self.download_hub.lock().map_err(unexpected_err)?;
            current_download_hub.replace(download_hub);
        }

        // trigger model download according to new library
        {
            // TODO
        }

        /* init ai handler */
        {
            // init AI handler after library is loaded
            let ai_handler = AIHandler::new(self).map_err(|e| {
                tracing::error!(task = "init ai handler", "Failed: {}", e);
                rspc::Error::new(
                    rspc::ErrorCode::InternalServerError,
                    format!("Failed to init AI handler: {}", e),
                )
            })?;
            tracing::info!(task = "init ai handler", "Success");
            let mut current_ai_handler = self.ai_handler.lock().map_err(unexpected_err)?;
            current_ai_handler.replace(ai_handler);
        }

        {
            self.trigger_unfinished_tasks().await;
        }

        Ok(library)

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
}
