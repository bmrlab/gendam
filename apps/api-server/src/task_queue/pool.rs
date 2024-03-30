use crate::CtxWithLibrary;
use file_handler::video::{VideoHandler, VideoTaskType};
use prisma_lib::{asset_object, file_handler_task, PrismaClient};
use std::{
    collections::HashMap,
    sync::{
        mpsc::{self, Sender},
        Arc,
    },
};
use thread_priority::{ThreadBuilder, ThreadPriority};
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};

// TODO make this more generic
pub struct Task {
    pub task_type: VideoTaskType,
    pub asset_object_id: i32,
    pub prisma_client: Arc<PrismaClient>,
    pub handler: VideoHandler,
}

impl Task {
    async fn run(&self) -> anyhow::Result<()> {
        self.save_starts_at().await;
        match self.handler.run_task(self.task_type.clone()).await {
            Ok(()) => {
                self.save_ends_at(None).await;
            }
            Err(e) => {
                self.save_ends_at(Some(e.to_string())).await;
            }
        }

        Ok(())
    }

    async fn save_starts_at(&self) {
        self.prisma_client
            .file_handler_task()
            .update(
                file_handler_task::asset_object_id_task_type(
                    self.asset_object_id,
                    self.task_type.to_string(),
                ),
                vec![file_handler_task::starts_at::set(Some(
                    chrono::Utc::now().into(),
                ))],
            )
            .exec()
            .await
            .expect(&format!("failed save_starts_at {}", self.task_type));
    }

    async fn save_ends_at(&self, error: Option<String>) {
        let (exit_code, exit_message) = match error {
            Some(error) => (Some(2), Some(error)),
            None => (Some(0), None),
        };

        self.prisma_client
            .file_handler_task()
            .update(
                file_handler_task::asset_object_id_task_type(
                    self.asset_object_id,
                    self.task_type.to_string(),
                ),
                vec![
                    file_handler_task::ends_at::set(Some(chrono::Utc::now().into())),
                    file_handler_task::exit_code::set(exit_code),
                    file_handler_task::exit_message::set(exit_message),
                ],
            )
            .exec()
            .await
            .expect(&format!("failed save_ends_at {}", self.task_type));
    }
}

pub enum TaskPayload {
    Task(Task),
    CancelByTaskId(String),
    CancelByAssetId(String),
    CancelAll,
}

pub fn init_task_pool() -> anyhow::Result<Sender<TaskPayload>> {
    // task_mapping can be optimized to HashMap<String, HashMap<String, CancellationToken>>
    let task_mapping: HashMap<String, CancellationToken> = HashMap::new();
    let task_mapping = Arc::new(RwLock::new(task_mapping));
    let task_mapping_clone = task_mapping.clone();

    let (tx, rx) = mpsc::channel();
    let (task_tx, task_rx) = mpsc::channel::<Task>();

    let cancel_token = CancellationToken::new();
    let cloned_token = cancel_token.clone();

    tokio::spawn(async move {
        loop {
            match rx.recv() {
                Ok(payload) => match payload {
                    TaskPayload::Task(task) => {
                        let task_id = format!("{}-{}", task.asset_object_id, task.task_type);

                        info!("Task received: {}", task_id);

                        {
                            let current_cancel_token = CancellationToken::new();
                            task_mapping
                                .write()
                                .await
                                .insert(task_id.clone(), current_cancel_token.clone());
                        }

                        if let Err(e) = task_tx.send(task) {
                            error!("failed to send task: {}", e);
                        }
                    }
                    TaskPayload::CancelByTaskId(task_id) => {
                        if let Some(item) = task_mapping.read().await.get(&task_id) {
                            item.cancel();
                            info!("task {} set to canceled", task_id);
                        } else {
                            warn!("failed to find task: {}", task_id);
                        }
                    }
                    TaskPayload::CancelByAssetId(asset_object_id) => {
                        task_mapping.read().await.iter().for_each(|(key, token)| {
                            if key.starts_with(&asset_object_id) {
                                token.cancel();
                                info!("task {} set to canceled", key);
                            }
                        });
                    }
                    TaskPayload::CancelAll => {
                        cancel_token.cancel();
                    }
                },
                _ => {}
            }
        }
    });

    match ThreadBuilder::default()
        .priority(ThreadPriority::Min)
        .spawn(move |result| {
            if let Err(e) = result {
                warn!("failed to set priority: {}", e);
            }

            match tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
            {
                Ok(rt) => {
                    rt.block_on(async move {
                        loop {
                            match task_rx.recv() {
                                Ok(task) => {
                                    let task_id =
                                        format!("{}-{}", task.asset_object_id, task.task_type);

                                    info!("Task processing: {}", task_id);

                                    let current_cancel_token = task_mapping_clone
                                        .read()
                                        .await
                                        .get(&task_id)
                                        .expect("Error creating task: failed to find current_cancel_token")
                                        .clone();

                                    tokio::select! {
                                        _ = task.run() => {
                                            info!("task {} finished", task_id);
                                        }
                                        _ = current_cancel_token.cancelled() => {
                                            info!("task {} canceled by Cancel", task_id);
                                        }
                                        _ = cloned_token.cancelled() => {
                                            info!("task {} canceled by CancelAll", task_id);
                                        }
                                    }

                                    // remove data in task_mapping
                                    task_mapping_clone.write().await.remove(&task_id);

                                }
                                _ => {
                                    error!("No Task Error");
                                    // task_pool will be dropped when library changed
                                    // so just break here
                                    // break;
                                }
                            }
                        }
                    });
                }
                Err(e) => {
                    error!("failed to build tokio runtime: {}", e);
                }
            };
        }) {
        Ok(thread) => {
            info!("Task pool thread created: {:?}", thread.thread().id(),);
        }
        Err(e) => {
            error!("failed to build thread: {}", e);
        }
    };

    Ok(tx)
}

pub async fn create_video_task(
    asset_object_data: &asset_object::Data,
    ctx: &impl CtxWithLibrary,
) -> Result<(), ()> {
    let library = &ctx.library().map_err(|e| {
        error!(
            "library must be set before triggering create_video_task: {}",
            e
        );
    })?;

    let local_video_file_full_path = format!(
        "{}/{}",
        library.files_dir.to_str().unwrap(),
        asset_object_data.hash
    );

    let ai_handler = ctx.get_ai_handler();

    let video_handler = match VideoHandler::new(
        local_video_file_full_path,
        &asset_object_data.hash,
        &library,
    ) {
        Ok(vh) => vh
            .with_clip(ai_handler.clip)
            .with_blip(ai_handler.blip)
            .with_whisper(ai_handler.whisper),
        Err(e) => {
            error!("failed to initialize video handler: {}", e);
            return Err(());
        }
    };

    for task_type in video_handler.get_supported_task_types() {
        let x = library
            .prisma_client()
            .file_handler_task()
            .upsert(
                file_handler_task::asset_object_id_task_type(
                    asset_object_data.id,
                    task_type.to_string(),
                ),
                file_handler_task::create(asset_object_data.id, task_type.to_string(), vec![]),
                vec![
                    file_handler_task::starts_at::set(None),
                    file_handler_task::ends_at::set(None),
                    file_handler_task::exit_code::set(None),
                    file_handler_task::exit_message::set(None),
                ],
            )
            .exec()
            .await;

        match x {
            Ok(res) => {
                info!("Task created: {:?}", res);
            }
            Err(e) => {
                error!("Failed to create task: {}", e);
            }
        }
    }

    let tx = ctx.get_task_tx();

    match tx.lock() {
        Ok(tx) => {
            for task_type in video_handler.get_supported_task_types() {
                let vh = video_handler.clone();
                let task_type_clone = task_type.clone();
                let asset_object_id = asset_object_data.id;
                let prisma_client = library.prisma_client();

                match tx.send(TaskPayload::Task(Task {
                    handler: vh,
                    task_type: task_type_clone.clone(),
                    asset_object_id,
                    prisma_client,
                })) {
                    Ok(_) => {
                        info!("Task queued {}", asset_object_data.hash);
                    }
                    Err(e) => {
                        error!("Failed to queue task {}: {}", asset_object_data.hash, e);
                    }
                }
            }
        }
        Err(e) => {
            error!("Failed to lock mutex: {}", e);
        }
    }

    Ok(())
}
