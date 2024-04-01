use super::priority::TaskPriority;
use file_handler::video::{VideoHandler, VideoTaskType};
use priority_queue::PriorityQueue;
use prisma_lib::{file_handler_task, PrismaClient};
use std::{
    collections::HashMap,
    hash::Hash,
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
#[derive(Clone)]
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

impl Hash for Task {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.asset_object_id.hash(state);
        self.task_type.hash(state);
    }
}

impl PartialEq for Task {
    fn eq(&self, other: &Self) -> bool {
        self.asset_object_id == other.asset_object_id && self.task_type == other.task_type
    }
}

impl Eq for Task {}

pub enum TaskPayload {
    Task((Task, TaskPriority)),
    CancelByAssetAndType(i32, VideoTaskType),
    CancelByAssetId(i32),
    CancelAll,
}

pub fn init_task_pool() -> anyhow::Result<Sender<TaskPayload>> {
    let task_mapping: HashMap<i32, HashMap<VideoTaskType, CancellationToken>> = HashMap::new();
    let task_mapping = Arc::new(RwLock::new(task_mapping));
    let task_mapping_clone = task_mapping.clone();

    let task_queue = PriorityQueue::new();
    let task_queue = Arc::new(RwLock::new(task_queue));
    let task_queue_clone = task_queue.clone();

    let current_task_priority = Arc::new(RwLock::new(None));
    let current_task_priority_clone = current_task_priority.clone();

    let (tx, rx) = mpsc::channel();

    let (task_tx, task_rx) = tokio::sync::mpsc::channel::<()>(512);
    let (priority_tx, priority_rx) = tokio::sync::mpsc::channel::<()>(512);

    let asset_cancel_token = CancellationToken::new();
    let asset_cancel_token_clone = asset_cancel_token.clone();

    // 监听一个 AssetObject 需要处理，加入队列后，由 task_rx 继续处理每个类型的子任务
    tokio::spawn(handle_task_payload_input(
        task_queue,
        task_mapping,
        current_task_priority,
        rx,
        task_tx,
        priority_tx,
        asset_cancel_token,
    ));

    /*
     * 在一个较低优先级的线程中执行 loop_until_queue_empty
     * 但是，感觉不大对，这么写并不会被降低优先级，需要再仔细研究下
     */
    match ThreadBuilder::default()
        .priority(ThreadPriority::Min)
        .spawn(move |result| {
            if let Err(e) = result {
                warn!("failed to set priority: {}", e);
            }
            // 如果要保证 `loop_until_queue_empty` 在一个低优先级的线程中进行
            // 这里应该用 `new_current_thread`
            // 这里用了 `multi_thread`，应该无法保证线程优先级
            match tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
            {
                Ok(rt) => {
                    // 监听一个 AssetObject 的子任务
                    rt.block_on(loop_until_queue_empty(
                        task_queue_clone,
                        task_mapping_clone,
                        current_task_priority_clone,
                        task_rx,
                        priority_rx,
                        asset_cancel_token_clone,
                    ));
                }
                Err(e) => {
                    error!("Failed to build tokio runtime: {}", e);
                }
            };
        }) {
        Ok(thread) => {
            info!("Task pool thread created: {:?}", thread.thread().id());
        }
        Err(e) => {
            error!("Failed to build thread: {}", e);
        }
    };

    Ok(tx)
}

/// 监听来自外部的任务创建、任务取消
/// 收到任务则将任务加入队列
async fn handle_task_payload_input(
    task_queue: Arc<RwLock<PriorityQueue<Task, TaskPriority>>>,
    task_mapping: Arc<RwLock<HashMap<i32, HashMap<VideoTaskType, CancellationToken>>>>,
    current_task_priority: Arc<RwLock<Option<TaskPriority>>>,
    rx: mpsc::Receiver<TaskPayload>,
    task_tx: tokio::sync::mpsc::Sender<()>,
    priority_tx: tokio::sync::mpsc::Sender<()>,
    asset_cancel_token: CancellationToken,
) {
    loop {
        match rx.recv() {
            Ok(payload) => match payload {
                TaskPayload::Task((task, priority)) => {
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

                    {
                        let mut task_queue = task_queue.write().await;
                        // 通过 insert_order 确保在相同优先级和时间戳的情况下，先加入的任务优先级相对更高
                        let priority = priority.with_insert_order(task_queue.len());
                        task_queue.push(task, priority);

                        let current_priority = current_task_priority.read().await;
                        match *current_priority {
                            Some(current_priority) => {
                                if priority > current_priority {
                                    if let Err(e) = priority_tx.send(()).await {
                                        error!("failed to send priority: {}", e);
                                    }
                                }
                            }
                            _ => {}
                        }
                    }

                    if let Err(e) = task_tx.send(()).await {
                        error!("failed to send task: {}", e);
                    }
                }
                TaskPayload::CancelByAssetAndType(asset_object_id, task_type) => {
                    let task_mapping = task_mapping.read().await;
                    if let Some(item) = task_mapping.get(&asset_object_id) {
                        if let Some(cancel_token) = item.get(&task_type) {
                            cancel_token.cancel();
                        } else {
                            warn!(
                                "Task not found for asset obejct {} of type {}",
                                asset_object_id, task_type
                            );
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
}

/// 持续处理任务队列中的任务
/// 当任务队列为空时，如果 task_rx 收到信息，则继续处理任务
async fn loop_until_queue_empty(
    task_queue: Arc<RwLock<PriorityQueue<Task, TaskPriority>>>,
    task_mapping: Arc<RwLock<HashMap<i32, HashMap<VideoTaskType, CancellationToken>>>>,
    current_task_priority: Arc<RwLock<Option<TaskPriority>>>,
    mut task_rx: tokio::sync::mpsc::Receiver<()>,
    mut priority_rx: tokio::sync::mpsc::Receiver<()>,
    asset_cancel_token: CancellationToken,
) {
    // TODO 优化这里的两层循环
    loop {
        // 等待 task_rx 收到任务
        match task_rx.recv().await {
            Some(_) => {
                loop {
                    // 持续处理队列中的任务直到为空
                    if task_queue.read().await.len() == 0 {
                        break;
                    }

                    let (task, task_priority) = {
                        let mut task_queue = task_queue.write().await;
                        let top_task = task_queue.pop().unwrap();
                        let mut current_task_priority = current_task_priority.write().await;
                        *current_task_priority = Some(top_task.1);
                        top_task
                    };

                    let asset_object_id = task.asset_object_id;
                    let task_type = task.task_type.clone();
                    info!(
                        "Task processing: {} {}, priority: {}",
                        asset_object_id, task_type, task_priority
                    );

                    /*
                     * 这里专门用 inline 写，current_cancel_token 赋值完了以后直接释放 task_mapping，
                     * 不能改成 let 赋值给一个新变量存下来而且直接暴露在这个 fn 的上下文里，
                     * 不然下面的 task_mapping_clone.write().await 会死锁，
                     * 而且 current_cancel_token 不能是 & cancel_token.clone() 释放 task_mapping
                     */
                    let current_cancel_token = match task_mapping.read().await.get(&asset_object_id)
                    {
                        Some(item) => match item.get(&task_type) {
                            Some(cancel_token) => cancel_token.clone(),
                            None => {
                                error!(
                                    "No task in the queue for asset obejct {} of type {}",
                                    asset_object_id, task_type
                                );
                                continue;
                            }
                        },
                        None => {
                            error!(
                                "No tasks in the queue for asset obejct {} ",
                                asset_object_id
                            );
                            continue;
                        }
                    };

                    let task_clone = task.clone();
                    let mut cancel_by_priority = false;

                    tokio::select! {
                        _ = task.run() => {
                            info!("Task finished: {} {}", asset_object_id, task_type);
                        }
                        _ = current_cancel_token.cancelled() => {
                            info!("Task canceled: {} {}", asset_object_id, task_type);
                        }
                        _ = asset_cancel_token.cancelled() => {
                            info!("Task canceled by CancelAll: {}", asset_object_id);
                        }
                        Some(()) = priority_rx.recv() => {
                            // 收到一个优先级更高的任务，当前任务就放弃掉
                            // TODO 实际上这里应该 “暂停” 任务，但是没找到特别好的写法
                            //
                            // 有两个思路：
                            // - 如果可行的话，实现 future 的暂停，应该需要操作到 tokio runtime；问题是 tokio 原生没有支持，也没有很好的社区实现
                            // - 退而求其次的话，任务执行时跳过已执行的部分，这个要求任务记录状态在数据库中；问题是会浪费很多计算资源
                            info!("Task with higher priority arrived: {} {}", asset_object_id, task_type);
                            cancel_by_priority = true;
                        }
                    }

                    if cancel_by_priority {
                        // current task need to be preserved
                        let mut task_queue = task_queue.write().await;
                        task_queue.push(
                            task_clone,
                            current_task_priority
                                .read()
                                .await
                                .clone()
                                .expect("priority not set correctly"),
                        );
                    } else {
                        // remove data in task_mapping
                        // task_mapping_clone.write().await.remove(&task_id);
                        let mut task_mapping = task_mapping.write().await;
                        match task_mapping.get_mut(&asset_object_id) {
                            Some(item) => {
                                item.remove(&task_type);
                                if item.is_empty() {
                                    task_mapping.remove(&asset_object_id);
                                }
                            }
                            None => {} // 前面已经读取到过了, 这里不可能 None, 真的遇到了忽略也没有问题
                        };
                    }

                    // 任务执行完了，那么当前任务优先级设置为None
                    {
                        *current_task_priority.write().await = None;
                    }
                }
            }
            _ => {
                error!("No Task Error");
            }
        }
    }
}
