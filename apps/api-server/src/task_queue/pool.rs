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
    CancelByAssetAndType(i32, VideoTaskType),
    CancelByAssetId(i32),
    CancelAll,
}

pub fn init_task_pool() -> anyhow::Result<Sender<TaskPayload>> {
    let task_mapping: HashMap<i32, HashMap<VideoTaskType, CancellationToken>> = HashMap::new();
    let task_mapping = Arc::new(RwLock::new(task_mapping));
    let task_mapping_clone = task_mapping.clone();

    let (tx, rx) = mpsc::channel();
    let (task_tx, task_rx) = mpsc::channel::<Task>();

    let asset_cancel_token = CancellationToken::new();
    let asset_cancel_token_clone = asset_cancel_token.clone();

    // 监听一个 AssetObject 需要处理，加入队列后，由 task_rx 继续处理每个类型的子任务
    tokio::spawn(async move {
        loop {
            match rx.recv() {
                Ok(payload) => match payload {
                    TaskPayload::Task(task) => {
                        let asset_object_id = task.asset_object_id;
                        let task_type = task.task_type.clone();
                        info!("Task received: {} {}", asset_object_id, task_type);
                        {
                            let current_cancel_token = CancellationToken::new();
                            let mut task_mapping = task_mapping.write().await;
                            match task_mapping.get_mut(&asset_object_id) {
                                Some(item) => {
                                    item.insert(task_type, current_cancel_token);
                                }
                                None => {
                                    let mut new_item = HashMap::new();
                                    new_item.insert(task_type, current_cancel_token);
                                    task_mapping.insert(asset_object_id, new_item);
                                }
                            };
                        }
                        if let Err(e) = task_tx.send(task) {
                            error!("failed to send task: {}", e);
                        }
                    }
                    TaskPayload::CancelByAssetAndType(asset_object_id, task_type) => {
                        let task_mapping = task_mapping.read().await;
                        if let Some(item) = task_mapping.get(&asset_object_id) {
                            if let Some(cancel_token) = item.get(&task_type) {
                                cancel_token.cancel();
                            } else {
                                warn!("Task not found for asset obejct {} of type {}", asset_object_id, task_type);
                            }
                            info!("Task cancelled {} {}", asset_object_id, task_type);
                        } else {
                            warn!("Task not found for asset obejct {}", asset_object_id);
                        }
                    }
                    TaskPayload::CancelByAssetId(asset_object_id) => {
                        let task_mapping = task_mapping.read().await;
                        if let Some(item) = task_mapping.get(&asset_object_id) {
                            item.iter().for_each(|(task_type, cancel_token)| {
                                cancel_token.cancel();
                                info!("Task cancelled {} {}", asset_object_id, task_type);
                            });
                        } else {
                            warn!("Task not found for asset obejct {}", asset_object_id);
                        }
                    }
                    TaskPayload::CancelAll => {
                        asset_cancel_token.cancel();
                    }
                },
                _ => {}
            }
        }
    });

    // 监听一个 AssetObject 的子任务
    let loop_for_next_single_task = async move {
        loop {
            match task_rx.recv() {
                Ok(task) => {
                    let asset_object_id = task.asset_object_id;
                    let task_type = task.task_type.clone();
                    info!("Task processing: {} {}", asset_object_id, task_type);

                    /*
                     * 这里专门用 inline 写，current_cancel_token 赋值完了以后直接释放 task_mapping，
                     * 不能改成 let 赋值给一个新变量存下来而且直接暴露在这个 fn 的上下文里，
                     * 不然下面的 task_mapping_clone.write().await 会死锁，
                     * 而且 current_cancel_token 不能是 & cancel_token.clone() 释放 task_mapping
                     */
                    let current_cancel_token =
                        match task_mapping_clone.read().await.get(&asset_object_id)
                    {
                        Some(item) => match item.get(&task_type) {
                            Some(cancel_token) => cancel_token.clone(),
                            None => {
                                error!("No task in the queue for asset obejct {} of type {}", asset_object_id, task_type);
                                continue;
                            }
                        },
                        None => {
                            error!("No tasks in the queue for asset obejct {} ", asset_object_id);
                            continue;
                        }
                    };

                    tokio::select! {
                        _ = task.run() => {
                            info!("Task finished: {} {}", asset_object_id, task_type);
                        }
                        _ = current_cancel_token.cancelled() => {
                            info!("Task canceled: {} {}", asset_object_id, task_type);
                        }
                        _ = asset_cancel_token_clone.cancelled() => {
                            info!("Task canceled by CancelAll: {}", asset_object_id);
                        }
                    }

                    // remove data in task_mapping
                    // task_mapping_clone.write().await.remove(&task_id);
                    let mut task_mapping = task_mapping_clone.write().await;
                    match task_mapping.get_mut(&asset_object_id) {
                        Some(item) => {
                            item.remove(&task_type);
                            if item.is_empty() {
                                task_mapping.remove(&asset_object_id);
                            }
                        }
                        None => {}  // 前面已经读取到过了, 这里不可能 None, 真的遇到了忽略也没有问题
                    };
                }
                _ => {
                    error!("No Task Error");
                    // task_pool will be dropped when library changed
                    // so just break here
                    // break;
                }
            }
        }
    };

    /*
     * 在一个较低优先级的线程中执行 loop_for_next_single_task
     * 但是，感觉不大对，这么写并不会被降低优先级，需要再仔细研究下
     */
    match ThreadBuilder::default().priority(ThreadPriority::Min).spawn(
        move |result| {
            if let Err(e) = result {
                warn!("failed to set priority: {}", e);
            }
            match tokio::runtime::Builder::new_multi_thread().enable_all().build() {
                Ok(rt) => {
                    rt.block_on(loop_for_next_single_task);
                }
                Err(e) => {
                    error!("Failed to build tokio runtime: {}", e);
                }
            };
        }
    ) {
        Ok(thread) => {
            info!("Task pool thread created: {:?}", thread.thread().id());
        }
        Err(e) => {
            error!("Failed to build thread: {}", e);
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

    tracing::debug!("asset object: {:?}", asset_object_data);

    let video_has_audio = match asset_object_data.media_data() {
        Ok(Some(metadata)) => metadata.has_audio,
        _ => {
            None
        }
    };

    for task_type in video_handler.get_supported_task_types(video_has_audio) {
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
            for task_type in video_handler.get_supported_task_types(video_has_audio) {
                match tx.send(TaskPayload::Task(Task {
                    handler: video_handler.clone(),
                    task_type: task_type.clone(),
                    asset_object_id: asset_object_data.id,
                    prisma_client: library.prisma_client(),
                })) {
                    Ok(_) => {
                        info!("Task queued {} {}", asset_object_data.id, &task_type);
                    }
                    Err(e) => {
                        error!("Failed to queue task {} {}: {}", asset_object_data.id, &task_type, e);
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
