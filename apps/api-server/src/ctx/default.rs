// Ctx 和 Store 的默认实现，主要给 api_server/main 用，不过目前 CtxWithLibrary 的实现也是可以给 tauri 用的，就先用着
use super::traits::{
    CtxError, CtxStore, CtxWithAI, CtxWithDownload, CtxWithLibrary, CtxWithP2P, StoreError,
};
use crate::cron_jobs::delete_unlinked_assets;
use crate::{
    ai::AIHandler,
    download::{DownloadHub, DownloadReporter, DownloadStatus},
    routes::{assets::process::build_content_index, p2p::info::ShareInfo},
};
use async_trait::async_trait;
use content_base::{ContentBase, ContentBaseCtx};
use content_library::{load_library, Library};
use futures::FutureExt;
use p2p::Node;
use std::{
    boxed::Box,
    path::PathBuf,
    sync::{atomic::AtomicBool, Arc, Mutex},
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

pub struct Ctx<S: CtxStore> {
    is_busy: Arc<Mutex<AtomicBool>>, // is loading or unloading a library
    local_data_root: PathBuf,
    resources_dir: PathBuf,
    temp_dir: PathBuf,
    cache_dir: PathBuf,
    store: Arc<Mutex<S>>,
    current_library: Arc<Mutex<Option<Library>>>,
    content_base: Arc<Mutex<Option<ContentBase>>>,
    ai_handler: Arc<Mutex<Option<AIHandler>>>,
    download_hub: Arc<Mutex<Option<DownloadHub>>>,
    node: Arc<Mutex<Node<ShareInfo>>>,
    cron: Arc<tokio::sync::Mutex<cron::Instance>>,
}

impl<S: CtxStore> Clone for Ctx<S> {
    fn clone(&self) -> Self {
        Self {
            local_data_root: self.local_data_root.clone(),
            resources_dir: self.resources_dir.clone(),
            store: self.store.clone(),
            current_library: self.current_library.clone(),
            content_base: self.content_base.clone(),
            ai_handler: self.ai_handler.clone(),
            is_busy: self.is_busy.clone(),
            download_hub: self.download_hub.clone(),
            temp_dir: self.temp_dir.clone(),
            cache_dir: self.cache_dir.clone(),
            node: Arc::clone(&self.node),
            cron: Arc::clone(&self.cron),
        }
    }
}

/*
 * is_busy 需要被实现一个 Drop trait，用于在离开作用域时自动释放锁
 * 不然当 load_library 和 unload_library 出错时，is_busy 会一直处于 true
 */
struct BusyGuard {
    pub is_busy: std::sync::Arc<std::sync::Mutex<std::sync::atomic::AtomicBool>>,
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
    pub fn new(
        local_data_root: PathBuf,
        resources_dir: PathBuf,
        temp_dir: PathBuf,
        cache_dir: PathBuf,
        store: Arc<Mutex<S>>,
        node: Arc<Mutex<Node<ShareInfo>>>,
    ) -> Self {
        Self {
            local_data_root,
            resources_dir,
            temp_dir,
            cache_dir,
            store,
            current_library: Arc::new(Mutex::new(None)),
            content_base: Arc::new(Mutex::new(None)),
            ai_handler: Arc::new(Mutex::new(None)),
            is_busy: Arc::new(Mutex::new(AtomicBool::new(false))),
            download_hub: Arc::new(Mutex::new(None)),
            node,
            cron: Arc::new(tokio::sync::Mutex::new(cron::Instance::init())),
        }
    }
}

impl<S: CtxStore + Send> CtxWithP2P for Ctx<S> {
    fn node(&self) -> Result<Node<ShareInfo>, CtxError> {
        let node = self.node.lock()?;
        Ok(node.clone())
    }
}

impl<S: CtxStore + Send> CtxWithAI for Ctx<S> {
    fn ai_handler(&self) -> Result<AIHandler, CtxError> {
        let ai_handler = self.ai_handler.lock()?;
        match ai_handler.as_ref() {
            Some(ai_handler) => Ok(ai_handler.clone()),
            None => Err(CtxError::BadRequest("No ai handler is set".into())),
        }
    }

    fn ai_handler_mutex(&self) -> Arc<Mutex<Option<AIHandler>>> {
        self.ai_handler.clone()
    }
}

impl<S: CtxStore + Send> CtxWithDownload for Ctx<S> {
    fn download_reporter(&self) -> Result<DownloadReporter, CtxError> {
        let download_hub = self.download_hub.lock()?;
        match download_hub.as_ref() {
            Some(download_hub) => Ok(download_hub.get_reporter()),
            None => Err(CtxError::BadRequest("No download reporter is set".into())),
        }
    }

    fn download_status(&self) -> Result<Vec<DownloadStatus>, CtxError> {
        let download_hub = self.download_hub.lock()?;
        match download_hub.as_ref() {
            Some(download_hub) => Ok(download_hub.get_file_list()),
            None => Err(CtxError::BadRequest("No download status is set".into())),
        }
    }
}

// lock()? will return CtxError
impl<T> From<std::sync::PoisonError<T>> for CtxError {
    fn from(e: std::sync::PoisonError<T>) -> Self {
        CtxError::Internal(format!("Lock poison error: {}", e))
    }
}

impl<S: CtxStore + Send> Ctx<S> {
    async fn trigger_unfinished_tasks(&self) -> () {
        let Ok(library) = self.library() else {
            return;
        };
        let asset_object_data_list = match library
            .prisma_client()
            .asset_object()
            .find_many(vec![prisma_lib::asset_object::tasks::some(vec![
                prisma_lib::file_handler_task::exit_code::equals(None),
            ])])
            .exec()
            .await
        {
            Ok(asset_object_data_list) => asset_object_data_list,
            Err(e) => {
                tracing::error!("Failed to fetch unfinished tasks: {}", e);
                return;
            }
        };
        tracing::info!(
            "Found {} assets with unfinished tasks",
            asset_object_data_list.len()
        );
        for asset_object_data in asset_object_data_list {
            if let Err(e) = build_content_index(&library, self, &asset_object_data.hash, true).await
            {
                tracing::error!(error = ?e, "Failed trigger content index rebuild for asset {}", asset_object_data.hash);
            }
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

    fn library(&self) -> Result<Library, CtxError> {
        match self.current_library.lock().unwrap().as_ref() {
            Some(library) => Ok(library.clone()),
            None => Err(CtxError::BadRequest("No current library is set".into())),
        }
    }

    fn library_id_in_store(&self) -> Option<String> {
        let store = self.store.lock().unwrap();
        store.get("current-library-id")
    }

    fn get_temp_dir(&self) -> PathBuf {
        self.temp_dir.clone()
    }

    fn get_cache_dir(&self) -> PathBuf {
        self.cache_dir.clone()
    }

    #[tracing::instrument(level = "info", skip_all)] // create a span for better tracking
    async fn unload_library(&self) -> Result<(), CtxError> {
        {
            let mut is_busy = self.is_busy.lock().unwrap();
            if *is_busy.get_mut() {
                // FIXME should use 429 too many requests error code
                return Err(CtxError::Conflict("App is busy".into()));
            }
            is_busy.store(true, std::sync::atomic::Ordering::Relaxed);
        }
        // guard 需要放在后面, 这样前面返回 app is busy 的时候就不会触发 guard 的 drop
        let _guard = BusyGuard {
            is_busy: self.is_busy.clone(),
        };

        /* cancel tasks */
        {
            let mut content_base = self.content_base.lock()?;
            *content_base = None;
            tracing::info!(task = "update content base", "Success");
        }

        /* shutdown ai handler */
        let current_ai_handler = {
            let ai_handler = self.ai_handler.lock()?;
            let ai_handler = ai_handler.as_ref().map(|v| v.clone());
            ai_handler
        };
        if let Some(ai_handler) = current_ai_handler {
            drop(ai_handler);
            tracing::info!(task = "shutdown ai handler", "Success");
        }

        /* update ctx */
        {
            let mut current_library = self.current_library.lock()?;
            *current_library = None; // same as self.current_library.lock().unwrap().take();
            tracing::info!(task = "update ctx", "Success");
        }

        /* update store */
        {
            // 其实这里可以不删掉 library-id 和 qdrant-id 的，因为下次 load_library 的时候会自动覆盖
            // 保留 unload 之前的数据，意想不到的问题少点, 所以下面直接注释掉了
            // let mut store = self.store.lock().map_err(unexpected_err)?;
            // let _ = store.delete("current-library-id");
            // let _ = store.delete("current-qdrant-pid");
            // if let Err(e) = store.save() {
            //     tracing::warn!(task = "update store", "Failed: {:?}", e);
            //     // this issue can be safely ignored
            // } else {
            //     tracing::info!(task = "update store", "Success");
            // }
        }

        // remove all cron job
        {
            let job_ids = self.cron.lock().await.get_all_job_id();
            for id in job_ids {
                let _ = self.cron.lock().await.delete_job(id).await;
            }
        }

        Ok(())
    }

    #[tracing::instrument(level = "info", skip(self))] // create a span for better tracking
    async fn load_library(&self, library_id: &str) -> Result<Library, CtxError> {
        {
            let mut is_busy = self.is_busy.lock().unwrap();
            if *is_busy.get_mut() {
                return Err(CtxError::Conflict("App is busy".into()));
            }
            is_busy.store(true, std::sync::atomic::Ordering::Relaxed);
        }
        // guard 需要放在后面, 这样前面返回 app is busy 的时候就不会触发 guard 的 drop
        let _guard = BusyGuard {
            is_busy: self.is_busy.clone(),
        };

        if let Some(library) = self.current_library.lock().unwrap().as_ref() {
            if library.id == library_id {
                tracing::info!("Library with id {} is already loaded", library_id);
                return Ok(library.clone());
            } else {
                return Err(CtxError::BadRequest(format!(
                    "Library with diffrerent id {} is already loaded",
                    library.id
                )));
            }
        }

        /* init library */
        let library = {
            let library = load_library(&self.local_data_root, &library_id)
                .await
                .map_err(|e| {
                    tracing::error!(task = "init library", "Failed: {:?}", e);
                    CtxError::Internal(format!("Failed to init library: {:?}", e))
                })?;
            tracing::info!(task = "init library", "Success");
            library
        };

        /* update ctx */
        {
            // should update ctx before self.qdrant_info() is called
            let mut current_library = self.current_library.lock()?;
            current_library.replace(library.clone());
            tracing::info!(task = "update ctx", "Success");
        }

        /* update store */
        {
            let mut store = self.store.lock()?;
            let _ = store.insert("current-library-id", library_id);
            if let Err(e) = store.save() {
                tracing::warn!(task = "update store", "Failed: {:?}", e);
                // this issue can be safely ignored
            } else {
                tracing::info!(task = "update store", "Success");
            }
        }

        // init download hub
        {
            let download_hub = DownloadHub::new();
            let mut current_download_hub = self.download_hub.lock()?;
            current_download_hub.replace(download_hub);
        }

        // trigger model download according to new library
        {
            // TODO
        }

        /* init ai handler */
        let ai_handler = {
            // init AI handler after library is loaded
            let ai_handler = AIHandler::new(self).map_err(|e| {
                tracing::error!(task = "init ai handler", "Failed: {}", e);
                CtxError::Internal(format!("Failed to init AI handler: {}", e))
            })?;
            tracing::info!(task = "init ai handler", "Success");
            let mut current_ai_handler = self.ai_handler.lock()?;
            current_ai_handler.replace(ai_handler.clone());
            ai_handler
        };

        /* init content base */
        let content_base = {
            let cb_ctx = ContentBaseCtx::new(&library.artifacts_dir_name(), &self.temp_dir)
                .with_audio_transcript(
                    Arc::new(ai_handler.audio_transcript.0),
                    &ai_handler.audio_transcript.1,
                )
                .with_llm(
                    Arc::new(ai_handler.llm.0),
                    &ai_handler.llm.1, // this comment is just for for alignment and better readability
                )
                .with_text_tokenizer(
                    Arc::new(ai_handler.text_tokenizer.0),
                    &ai_handler.text_tokenizer.1,
                )
                .with_multi_modal_embedding(
                    Arc::new(ai_handler.multi_modal_embedding.0),
                    &ai_handler.multi_modal_embedding.1,
                )
                .with_text_embedding(
                    Arc::new(ai_handler.text_embedding.0),
                    &ai_handler.text_embedding.1,
                )
                .with_image_caption(
                    Arc::new(ai_handler.image_caption.0),
                    &ai_handler.image_caption.1,
                );
            // 这个 block 后面不再使用 ai_handler 了，上面 with 函数里不需要 clone 直接 move 就行
            let cb = ContentBase::new(&cb_ctx, library.db()).map_err(|e| {
                tracing::error!(task = "init content base", "Failed: {}", e);
                CtxError::Internal(format!("Failed to init content base: {}", e))
            })?;
            tracing::info!(task = "init task pool", "Success");
            let mut current_cb = self.content_base.lock()?;
            current_cb.replace(cb.clone());

            cb
        };

        /* trigger unfinished tasks */
        {
            self.trigger_unfinished_tasks().await;
            tracing::info!(task = "trigger unfinished tasks", "Success");
        }

        // init cron
        {
            // 添加 定期删除未引用的assetobject任务
            let library_clone = library.clone();
            let content_base = content_base.clone();

            let task = cron::Task {
                title: Some("Delete unreferenced assetobject tasks periodically".to_string()),
                description: Some("Delete unreferenced assetobject tasks periodically".to_string()),
                enabled: true,
                id: uuid::Uuid::new_v4(),
                job_id: None,
                // cron: "0 0 */1 * * ?".to_string(), // 每小时
                cron: "0/10 * * * * ?".to_string(), // 测试 每10秒
                // job_fn: None
                job_fn: cron::create_job_fn(move || {
                    let library_arc = Arc::new(library_clone.clone());
                    let content_base_arc = Arc::new(content_base.clone());
                    async move { delete_unlinked_assets(library_arc, content_base_arc).await }
                        .boxed()
                }),
            };

            let _ = self.add_cron_task(task).await?;
        }

        Ok(library)
    }

    fn content_base(&self) -> Result<ContentBase, CtxError> {
        match self.content_base.lock().unwrap().as_ref() {
            Some(content_base) => Ok(content_base.clone()),
            None => Err(CtxError::BadRequest("No content base is set".into())),
        }
    }
}

impl<S: CtxStore + Send> Ctx<S> {
    async fn add_cron_task(&self, task: cron::Task) -> Result<(), CtxError> {
        let _ = self.cron.lock().await.create_job(task).await.map_err(|e| {
            tracing::error!(task = "add task", "Failed: {:?}", e);
            CtxError::Internal(format!("cron add task error: {:?}", e))
        })?;
        Ok(())
    }
}
