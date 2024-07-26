use crate::{
    payload::{Task, TaskPayload},
    priority::{OrderedTaskPriority, TaskPriority},
};
use content_base_context::ContentBaseCtx;
use content_base_task::{ContentTask, ContentTaskType, FileInfo};
use priority_queue::PriorityQueue;
use std::{
    collections::HashMap,
    path::Path,
    sync::{
        mpsc::{self, Sender},
        Arc,
    },
};
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};

#[derive(Clone)]
pub struct TaskPool {
    tx: Sender<TaskPayload>,
}

impl TaskPool {
    pub fn new(content_base: &ContentBaseCtx) -> anyhow::Result<Self> {
        let task_mapping: HashMap<String, HashMap<ContentTaskType, CancellationToken>> =
            HashMap::new();
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

        // 如果要保证 `loop_until_queue_empty` 在一个低优先级的线程中进行
        // 这里应该用 `new_current_thread`
        // 这里用了 `multi_thread`，应该无法保证线程优先级
        let content_base_clone = content_base.clone();
        std::thread::spawn(move || {
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
                        content_base_clone,
                    ));
                }
                Err(e) => {
                    error!("Failed to build tokio runtime: {}", e);
                }
            };
        });

        Ok(Self { tx })
    }

    pub async fn add_task(
        &self,
        file_identifier: &str,
        file_path: impl AsRef<Path>,
        task: &ContentTaskType,
        priority: Option<TaskPriority>,
    ) -> anyhow::Result<()> {
        self.tx.send(TaskPayload::Task(
            file_identifier.to_string(),
            file_path.as_ref().to_path_buf(),
            task.clone(),
            priority.unwrap_or(TaskPriority::Normal),
        ))?;

        Ok(())
    }

    pub fn cancel_specific(
        &self,
        file_identifier: &str,
        task: &ContentTaskType,
    ) -> anyhow::Result<()> {
        self.tx.send(TaskPayload::CancelByIdAndType(
            file_identifier.to_string(),
            task.clone(),
        ))?;

        Ok(())
    }

    pub fn cancel_by_file(&self, file_identifier: &str) -> anyhow::Result<()> {
        self.tx
            .send(TaskPayload::CancelById(file_identifier.to_string()))?;

        Ok(())
    }

    pub fn cancel_all(&self) -> anyhow::Result<()> {
        self.tx.send(TaskPayload::CancelAll)?;

        Ok(())
    }
}

/// 监听来自外部的任务创建、任务取消
/// 收到任务则将任务加入队列
async fn handle_task_payload_input(
    task_queue: Arc<RwLock<PriorityQueue<Task, OrderedTaskPriority>>>,
    task_mapping: Arc<RwLock<HashMap<String, HashMap<ContentTaskType, CancellationToken>>>>,
    current_task_priority: Arc<RwLock<Option<OrderedTaskPriority>>>,
    rx: mpsc::Receiver<TaskPayload>,
    task_tx: tokio::sync::mpsc::Sender<()>,
    priority_tx: tokio::sync::mpsc::Sender<()>,
    asset_cancel_token: CancellationToken,
) {
    loop {
        match rx.recv() {
            Ok(payload) => match payload {
                TaskPayload::Task(file_identifier, file_path, task_type, priority) => {
                    let task_type = task_type.clone();
                    info!("Task received: {} {}", file_identifier, &task_type);

                    // if task exists, ignore it
                    // 这里 task_queue 是所有未执行的任务
                    // task_mapping 还包括了正在执行的任务，所以用它
                    if let Some(tasks) = task_mapping.read().await.get(&file_identifier) {
                        if let Some(_) = tasks.get(&task_type) {
                            continue;
                        }
                    }

                    // record task in hash map
                    {
                        let current_cancel_token = CancellationToken::new();
                        let mut task_mapping = task_mapping.write().await;
                        match task_mapping.get_mut(&file_identifier) {
                            Some(item) => {
                                item.insert(task_type.clone(), current_cancel_token);
                            }
                            None => {
                                let mut new_item = HashMap::new();
                                new_item.insert(task_type.clone(), current_cancel_token);
                                task_mapping.insert(file_identifier.clone(), new_item);
                            }
                        };
                    }

                    {
                        let mut task_queue = task_queue.write().await;
                        let priority: OrderedTaskPriority = priority.into();
                        // 通过 order 确保在相同优先级和时间戳的情况下，先加入的任务优先级相对更高
                        let priority = priority.with_insert_order(task_queue.len());
                        task_queue.push(
                            Task {
                                file_identifier: file_identifier.clone(),
                                file_path: file_path.clone(),
                                task_type,
                            },
                            priority,
                        );

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
                TaskPayload::CancelByIdAndType(file_identifier, task_type) => {
                    let task_mapping = task_mapping.read().await;
                    if let Some(item) = task_mapping.get(&file_identifier) {
                        if let Some(cancel_token) = item.get(&task_type) {
                            cancel_token.cancel();
                        } else {
                            warn!(
                                "Task not found for asset obejct {} of type {}",
                                file_identifier, task_type
                            );
                        }
                        info!("Task cancelled {} {}", file_identifier, task_type);
                    } else {
                        warn!("Task not found for asset obejct {}", file_identifier);
                    }
                }
                TaskPayload::CancelById(file_identifier) => {
                    let task_mapping = task_mapping.read().await;
                    if let Some(item) = task_mapping.get(&file_identifier) {
                        item.iter().for_each(|(task_type, cancel_token)| {
                            cancel_token.cancel();
                            info!("Task cancelled {} {}", file_identifier, task_type);
                        });
                    } else {
                        warn!("Task not found for asset obejct {}", file_identifier);
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
    task_queue: Arc<RwLock<PriorityQueue<Task, OrderedTaskPriority>>>,
    task_mapping: Arc<RwLock<HashMap<String, HashMap<ContentTaskType, CancellationToken>>>>,
    current_task_priority: Arc<RwLock<Option<OrderedTaskPriority>>>,
    mut task_rx: tokio::sync::mpsc::Receiver<()>,
    mut priority_rx: tokio::sync::mpsc::Receiver<()>,
    asset_cancel_token: CancellationToken,
    content_base: ContentBaseCtx,
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

                    let file_identifier = task.file_identifier.clone();
                    let task_type = task.task_type.clone();
                    info!(
                        "Task processing: {} {}, priority: {}",
                        file_identifier, task_type, task_priority
                    );

                    /*
                     * 这里专门用 inline 写，current_cancel_token 赋值完了以后直接释放 task_mapping，
                     * 不能改成 let 赋值给一个新变量存下来而且直接暴露在这个 fn 的上下文里，
                     * 不然下面的 task_mapping_clone.write().await 会死锁，
                     * 而且 current_cancel_token 不能是 & cancel_token.clone() 释放 task_mapping
                     */
                    let current_cancel_token = match task_mapping.read().await.get(&file_identifier)
                    {
                        Some(item) => match item.get(&task_type) {
                            Some(cancel_token) => cancel_token.clone(),
                            None => {
                                error!(
                                    "No task in the queue for asset obejct {} of type {}",
                                    file_identifier, task_type
                                );
                                continue;
                            }
                        },
                        None => {
                            error!(
                                "No tasks in the queue for asset obejct {} ",
                                file_identifier
                            );
                            continue;
                        }
                    };

                    let task_clone = task.clone();
                    let mut cancel_by_priority = false;

                    let file_info = FileInfo {
                        file_identifier: file_identifier.clone(),
                        file_path: task.file_path.clone(),
                    };

                    tokio::select! {
                        result = task.task_type.run(&file_info, &content_base) => {
                            match result {
                                Err(e) => {
                                    error!("Task error: {} {} {}", file_identifier, task_type, e);
                                }
                                _ => {
                                    info!("Task finished: {} {}", file_identifier, task_type);
                                }
                            }
                        }
                        _ = current_cancel_token.cancelled() => {
                            info!("Task canceled: {} {}", file_identifier, task_type);
                        }
                        _ = asset_cancel_token.cancelled() => {
                            info!("Task canceled by CancelAll: {}", file_identifier);
                        }
                        Some(()) = priority_rx.recv() => {
                            // 收到一个优先级更高的任务，当前任务就放弃掉
                            // TODO 实际上这里应该 “暂停” 任务，但是没找到特别好的写法
                            //
                            // 有两个思路：
                            // - 如果可行的话，实现 future 的暂停，应该需要操作到 tokio runtime；问题是 tokio 原生没有支持，也没有很好的社区实现
                            // - 退而求其次的话，任务执行时跳过已执行的部分，这个要求任务记录状态在数据库中；问题是会浪费很多计算资源
                            info!("Task with higher priority arrived: {} {}", file_identifier, task_type);
                            cancel_by_priority = true;
                        }
                    }

                    if cancel_by_priority {
                        // current task need to be preserved
                        let mut task_queue = task_queue.write().await;
                        if let Some(priority) = current_task_priority.read().await.clone() {
                            task_queue.push(task_clone, priority);
                        } else {
                            tracing::error!("priority not set correctly");
                        }
                        // task_queue.push(
                        //     task_clone,
                        //     current_task_priority.read().await.clone().expect("priority not set correctly"),
                        // );
                    } else {
                        // remove data in task_mapping
                        // task_mapping_clone.write().await.remove(&task_id);
                        let mut task_mapping = task_mapping.write().await;
                        match task_mapping.get_mut(&file_identifier) {
                            Some(item) => {
                                item.remove(&task_type);
                                if item.is_empty() {
                                    task_mapping.remove(&file_identifier);
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
