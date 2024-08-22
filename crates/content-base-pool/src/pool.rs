use crate::{
    payload::{NewTaskPayload, Task, TaskPayload},
    priority::OrderedTaskPriority,
    TaskNotification, TaskPriority, TaskStatus,
};
use content_base_context::ContentBaseCtx;
use content_base_task::{ContentTask, ContentTaskType, FileInfo};
use priority_queue::PriorityQueue;
use std::{collections::HashMap, path::Path, sync::Arc};
use tokio::sync::{
    mpsc::{self, Sender},
    Notify, RwLock, Semaphore,
};
use tokio_util::sync::CancellationToken;

#[derive(Clone, Debug)]
pub struct TaskPool {
    tx: mpsc::Sender<TaskPayload>,
}

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
struct TaskId {
    file_identifier: String,
    task_type: ContentTaskType,
}

#[derive(Clone)]
struct TaskInQueue {
    task: Task,
    priority: OrderedTaskPriority,
    cancel_token: CancellationToken,
    notifier: Option<mpsc::Sender<TaskNotification>>,
}

impl TaskInQueue {
    async fn cancel(&self) {
        self.cancel_token.cancel();

        if let Some(tx) = &self.notifier {
            if let Err(e) = tx
                .send(TaskNotification {
                    task_type: self.task.task_type.clone(),
                    status: TaskStatus::Cancelled,
                    message: None,
                })
                .await
            {
                tracing::error!("Failed to send task cancelled notification: {}", e);
            }
        }

        tracing::info!(
            "Task cancelled {} {}",
            &self.task.file_identifier,
            &self.task.task_type
        );
    }
}

#[derive(Clone)]
struct TaskPoolContext {
    #[allow(dead_code)]
    task_bound: TaskBound,
    task_queue: Arc<RwLock<PriorityQueue<Task, OrderedTaskPriority>>>,
    task_mapping: Arc<RwLock<HashMap<String, HashMap<ContentTaskType, TaskInQueue>>>>,
    task_priority: Arc<RwLock<HashMap<TaskId, OrderedTaskPriority>>>,
    semaphore: Arc<Semaphore>,
    notifier: Arc<Notify>,
    tx: Sender<TaskPayload>,
    // after the tasks in vec finished, the task itself continue to run
    task_subscription: Arc<RwLock<HashMap<TaskId, Vec<TaskId>>>>,
    // after task finished, trigger the tasks in vec
    task_dispatch: Arc<RwLock<HashMap<TaskId, Vec<TaskId>>>>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum TaskBound {
    IO,
    CPU,
}

impl TaskPool {
    /// Create a TaskPool. The concurrency is only for CPU bound tasks. And the IO bound tasks
    /// will have 2x concurrency as CPU bound tasks. By default the concurrency is 1 and this
    /// value is not recommended to set.
    pub fn new(content_base: &ContentBaseCtx, concurrency: Option<usize>) -> anyhow::Result<Self> {
        let (tx, mut rx) = mpsc::channel(512);

        let task_subscription = Arc::new(RwLock::new(HashMap::new()));
        let task_dispatch = Arc::new(RwLock::new(HashMap::new()));

        let cpu_task_ctx = TaskPoolContext {
            task_bound: TaskBound::CPU,
            task_queue: Arc::new(RwLock::new(PriorityQueue::new())),
            task_mapping: Arc::new(RwLock::new(HashMap::new())),
            task_priority: Arc::new(RwLock::new(HashMap::new())),
            semaphore: Arc::new(Semaphore::new(concurrency.unwrap_or(1))),
            notifier: Arc::new(Notify::new()),
            task_subscription: task_subscription.clone(),
            task_dispatch: task_dispatch.clone(),
            tx: tx.clone(),
        };

        let io_task_ctx = TaskPoolContext {
            task_bound: TaskBound::IO,
            task_queue: Arc::new(RwLock::new(PriorityQueue::new())),
            task_mapping: Arc::new(RwLock::new(HashMap::new())),
            task_priority: Arc::new(RwLock::new(HashMap::new())),
            semaphore: Arc::new(Semaphore::new(concurrency.unwrap_or(1) * 2)),
            notifier: Arc::new(Notify::new()),
            task_subscription: task_subscription.clone(),
            task_dispatch: task_dispatch.clone(),
            tx: tx.clone(),
        };

        let cpu_ctx_clone = cpu_task_ctx.clone();
        let io_ctx_clone = io_task_ctx.clone();

        // loop for message
        tokio::spawn(async move {
            while let Some(payload) = rx.recv().await {
                let task_mappings = vec![
                    cpu_ctx_clone.task_mapping.clone(),
                    io_ctx_clone.task_mapping.clone(),
                ];

                match payload {
                    TaskPayload::Task(NewTaskPayload {
                        file_identifier,
                        file_path,
                        task_type,
                        priority,
                        notifier,
                    }) => {
                        let task_type = task_type.clone();
                        tracing::info!("Task received: {} {}", file_identifier, &task_type);

                        let task_bound = get_task_bound(&task_type).await;

                        let task_ctx = match task_bound {
                            TaskBound::CPU => &cpu_ctx_clone,
                            TaskBound::IO => &io_ctx_clone,
                        };

                        // if task exists, ignore it
                        // 这里 task_queue 是所有未执行的任务
                        // task_mapping 还包括了正在执行的任务，所以用它
                        if let Some(tasks) =
                            task_ctx.task_mapping.read().await.get(&file_identifier)
                        {
                            if let Some(_) = tasks.get(&task_type) {
                                continue;
                            }
                        }

                        let task = Task {
                            file_identifier: file_identifier.clone(),
                            file_path: file_path.clone(),
                            task_type: task_type.clone(),
                        };

                        if let Some(tx) = notifier.clone() {
                            if let Err(_) = tx
                                .send(TaskNotification {
                                    task_type: task.task_type.clone(),
                                    status: TaskStatus::Init,
                                    message: None,
                                })
                                .await
                            {
                                tracing::error!("Failed to send task init notification");
                            }
                        }

                        // record task in hash map
                        {
                            let current_cancel_token = CancellationToken::new();
                            let task_in_queue = TaskInQueue {
                                task: task.clone(),
                                priority: priority.into(),
                                cancel_token: current_cancel_token.clone(),
                                notifier,
                            };
                            let mut task_mapping = task_ctx.task_mapping.write().await;
                            match task_mapping.get_mut(&file_identifier) {
                                Some(item) => {
                                    item.insert(task_type.clone(), task_in_queue);
                                }
                                None => {
                                    let mut new_item = HashMap::new();
                                    new_item.insert(task_type.clone(), task_in_queue);
                                    task_mapping.insert(file_identifier.clone(), new_item);
                                }
                            };
                        }

                        {
                            let mut task_queue = task_ctx.task_queue.write().await;
                            let priority: OrderedTaskPriority = priority.into();
                            // 通过 order 确保在相同优先级和时间戳的情况下，先加入的任务优先级相对更高
                            let priority = priority.with_insert_order(task_queue.len());
                            task_queue.push(task, priority);

                            // 处理任务优先级
                            let current_priority = priority;
                            if task_ctx.semaphore.available_permits() == 0 {
                                // 找到优先级低于当前任务的任务，并取消它
                                let task_priority = task_ctx.task_priority.write().await;

                                let mut min_task_id: Option<TaskId> = None;

                                for (task_id, priority) in task_priority.iter() {
                                    if priority < &current_priority {
                                        if let Some(_min_task_id) = &min_task_id {
                                            if priority < &task_priority[_min_task_id] {
                                                min_task_id = Some(task_id.clone());
                                            }
                                        } else {
                                            min_task_id = Some(task_id.clone());
                                        }
                                    }
                                }

                                if let Some(task_id) = &min_task_id {
                                    let mut task_mapping = task_ctx.task_mapping.write().await;
                                    if let Some(item) =
                                        task_mapping.get_mut(&task_id.file_identifier)
                                    {
                                        if let Some(task) = item.get(&task_id.task_type) {
                                            task.cancel().await;

                                            // 需要把任务重新添加回去
                                            let mut task_queue = task_ctx.task_queue.write().await;
                                            task_queue
                                                .push(task.task.clone(), task.priority.clone());
                                        }
                                    }
                                }
                            }
                        }

                        task_ctx.notifier.notify_one();
                    }
                    TaskPayload::CancelByIdAndType(file_identifier, task_type) => {
                        for task_mapping in task_mappings {
                            let task_mapping = task_mapping.read().await;
                            if let Some(item) = task_mapping.get(&file_identifier) {
                                if let Some(task) = item.get(&task_type) {
                                    task.cancel().await;
                                }
                            }
                        }
                    }
                    TaskPayload::CancelById(file_identifier) => {
                        for task_mapping in task_mappings {
                            let task_mapping = task_mapping.read().await;
                            if let Some(item) = task_mapping.get(&file_identifier) {
                                for (_, task) in item.iter() {
                                    task.cancel().await;
                                }
                            }
                        }
                    }
                    TaskPayload::CancelAll => {
                        for task_mapping in task_mappings {
                            let task_mapping = task_mapping.read().await;
                            for (_, item) in task_mapping.iter() {
                                for (_, task) in item.iter() {
                                    task.cancel().await;
                                }
                            }
                        }
                    }
                }
            }
        });

        // loop for task execution
        let cb = content_base.clone();
        tokio::spawn(async move {
            cpu_task_ctx.run(&cb).await;
        });

        let cb = content_base.clone();
        tokio::spawn(async move {
            io_task_ctx.run(&cb).await;
        });

        Ok(Self { tx })
    }

    pub async fn add_task(
        &self,
        file_identifier: &str,
        file_path: impl AsRef<Path>,
        task: impl Into<ContentTaskType>,
        priority: Option<TaskPriority>,
        notifier: Option<mpsc::Sender<TaskNotification>>,
    ) -> anyhow::Result<()> {
        let mut payload = NewTaskPayload::new(file_identifier, file_path, task.into());
        payload.with_priority(priority);
        payload.with_notifier(notifier);

        self.tx.send(TaskPayload::Task(payload)).await?;

        Ok(())
    }

    pub async fn cancel_specific(
        &self,
        file_identifier: &str,
        task: &ContentTaskType,
    ) -> anyhow::Result<()> {
        self.tx
            .send(TaskPayload::CancelByIdAndType(
                file_identifier.to_string(),
                task.clone(),
            ))
            .await?;

        Ok(())
    }

    pub async fn cancel_by_file(&self, file_identifier: &str) -> anyhow::Result<()> {
        self.tx
            .send(TaskPayload::CancelById(file_identifier.to_string()))
            .await?;

        Ok(())
    }

    pub async fn cancel_all(&self) -> anyhow::Result<()> {
        self.tx.send(TaskPayload::CancelAll).await?;

        Ok(())
    }
}

async fn get_task_bound(_task_type: &ContentTaskType) -> TaskBound {
    // TODO determine task bound by task type
    TaskBound::CPU
}

impl TaskPoolContext {
    async fn run(&self, content_base: &ContentBaseCtx) {
        loop {
            let permit = self.semaphore.acquire().await.expect("semaphore acquired");
            let task = self.task_queue.write().await.pop();

            if let Some((task, priority)) = task {
                let semaphore = self.semaphore.clone();

                // TODO 这里 clone current_task 不是明智之举
                // 应该直接从 task_mapping 中拿出来
                // 但是现在 task_mapping 会用于判断任务重复执行
                // 需要用新的数据结构来承载这些任务
                let current_task = match self.task_mapping.read().await.get(&task.file_identifier) {
                    Some(item) => match item.get(&task.task_type) {
                        Some(current_task) => current_task.clone(),
                        None => {
                            tracing::error!("task not found: {}", &task.file_identifier);
                            drop(permit);
                            continue;
                        }
                    },
                    _ => {
                        tracing::error!("task not found: {}", &task.file_identifier);
                        drop(permit);
                        continue;
                    }
                };

                let task_id = TaskId {
                    file_identifier: task.file_identifier.clone(),
                    task_type: task.task_type.clone(),
                };

                tracing::debug!("Task popped: {} {}", &task.file_identifier, &task.task_type);

                let deps = current_task.task.task_type.task_dependencies();
                // If task has dependencies, add dependencies to task queue,
                // and record them in subscription and dispatch.
                if deps.len() > 0 {
                    // 同时 lock subscription 和 dispatch
                    // 避免 deadlock
                    let mut task_subscription = self.task_subscription.write().await;
                    let mut task_dispatch = self.task_dispatch.write().await;
                    let mut task_priority = self.task_priority.write().await;

                    let subscription = task_subscription.get(&task_id);

                    match subscription {
                        Some(v) if v.len() == 0 => {
                            // 说明 deps 都已经完成了
                            task_subscription.remove(&task_id);
                        }
                        _ => {
                            let deps: Vec<TaskId> = deps
                                .iter()
                                .map(|v| TaskId {
                                    file_identifier: task.file_identifier.clone(),
                                    task_type: v.clone(),
                                })
                                .collect();

                            match task_subscription.get_mut(&task_id) {
                                Some(v) => {
                                    v.extend(deps.clone());
                                }
                                _ => {
                                    task_subscription.insert(task_id.clone(), deps.clone());
                                }
                            }
                            for dep in deps.iter() {
                                match task_dispatch.get_mut(&dep) {
                                    Some(v) => {
                                        v.push(task_id.clone());
                                    }
                                    _ => {
                                        task_dispatch.insert(dep.clone(), vec![task_id.clone()]);
                                    }
                                }

                                // add task back to queue
                                let mut payload = NewTaskPayload::new(
                                    &dep.file_identifier,
                                    &current_task.task.file_path.clone(),
                                    dep.task_type.clone(),
                                );
                                payload.with_priority(Some(current_task.priority.into()));
                                payload.with_notifier(current_task.notifier.clone());

                                if let Err(e) = self.tx.send(TaskPayload::Task(payload)).await {
                                    tracing::error!("Failed to add dependent task: {}", e);
                                }
                            }

                            // 不执行任务了
                            drop(permit);

                            // // 移除任务记录
                            // // FIXME duplicate code
                            // match task_mapping.get_mut(&task.file_identifier) {
                            //     Some(item) => {
                            //         item.remove(&task.task_type);
                            //         if item.is_empty() {
                            //             task_mapping.remove(&task.file_identifier);
                            //         }
                            //     }
                            //     None => {} // 前面已经读取到过了, 这里不可能 None, 真的遇到了忽略也没有问题
                            // }

                            task_priority.remove(&task_id);

                            continue;
                        }
                    }
                }

                // Here we can just forget the permit, and increase permit after task is finished
                // by calling `add_permits` on semaphore.
                permit.forget();

                let content_base = content_base.clone();
                let task_mapping = self.task_mapping.clone();
                let task_priority = self.task_priority.clone();
                let task_dispatch = self.task_dispatch.clone();
                let task_subscription = self.task_subscription.clone();
                let task_tx = self.tx.clone();

                tokio::spawn(async move {
                    {
                        let mut task_priority = task_priority.write().await;
                        task_priority.insert(task_id.clone(), priority);
                    }

                    let file_info = FileInfo {
                        file_identifier: task.file_identifier.clone(),
                        file_path: task.file_path.clone(),
                    };

                    if let Some(tx) = current_task.notifier.clone() {
                        if let Err(_) = tx
                            .send(TaskNotification {
                                task_type: task.task_type.clone(),
                                status: TaskStatus::Started,
                                message: None,
                            })
                            .await
                        {
                            tracing::error!("Failed to send task start notification");
                        }
                    }

                    tokio::select! {
                        result = task.task_type.run(&file_info, &content_base) => {
                            let notification = match &result {
                                Err(e) => {
                                    tracing::error!("Task error: {} {} {}", &task.file_identifier, &task.task_type, e);
                                    TaskNotification {
                                        task_type: task.task_type.clone(),
                                        status: TaskStatus::Error,
                                        message: Some(e.to_string()),
                                    }
                                }
                                _ => {
                                    tracing::info!("Task finished: {} {}", &task.file_identifier, &task.task_type);
                                    TaskNotification {
                                        task_type: task.task_type.clone(),
                                        status: TaskStatus::Finished,
                                        message: None,
                                    }
                                }
                            };

                            if let Some(tx) = current_task.notifier.clone() {
                                if let Err(_) = tx.send(notification).await {
                                    tracing::error!("Failed to send task result");
                                }
                            }
                        }
                        _ = current_task.cancel_token.cancelled() => {
                            tracing::info!("Spawned task has been cancelled: {} {}", &task.file_identifier, &task.task_type);
                        }
                    }

                    // add permit back
                    semaphore.add_permits(1);

                    {
                        let mut task_mapping = task_mapping.write().await;
                        match task_mapping.get_mut(&task.file_identifier) {
                            Some(item) => {
                                item.remove(&task.task_type);
                                if item.is_empty() {
                                    task_mapping.remove(&task.file_identifier);
                                }
                            }
                            None => {} // 前面已经读取到过了, 这里不可能 None, 真的遇到了忽略也没有问题
                        }
                    }

                    {
                        let mut task_priority = task_priority.write().await;
                        task_priority.remove(&task_id);
                    }

                    {
                        // 同时 lock subscription 和 dispatch
                        // 避免 deadlock
                        let mut task_subscription = task_subscription.write().await;
                        let mut task_dispatch = task_dispatch.write().await;

                        if let Some(targets) = task_dispatch.remove(&task_id) {
                            for target in targets.iter() {
                                match task_subscription.get_mut(target) {
                                    Some(v) => {
                                        v.retain(|x| x != &task_id);

                                        if v.is_empty() {
                                            let mut payload = NewTaskPayload::new(
                                                &target.file_identifier,
                                                &current_task.task.file_path,
                                                target.task_type.clone(),
                                            );
                                            payload
                                                .with_priority(Some(current_task.priority.into()));
                                            payload.with_notifier(current_task.notifier.clone());

                                            if let Err(e) =
                                                task_tx.send(TaskPayload::Task(payload)).await
                                            {
                                                tracing::error!("Failed to dispatch task: {}", e);
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                });
            } else {
                drop(permit);
            }
        }
    }
}
