use crate::{
    mapping::TaskStore,
    payload::{NewTaskPayload, Task, TaskId, TaskPayload},
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

#[derive(Clone)]
struct TaskInQueue {
    task: Arc<Task>,
    priority: OrderedTaskPriority,
    cancel_token: CancellationToken,
    notifier: Option<mpsc::Sender<TaskNotification>>,
}

impl TaskInQueue {
    async fn cancel(&self) {
        self.cancel_token.cancel();

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
    task_queue: Arc<RwLock<PriorityQueue<TaskId, OrderedTaskPriority>>>,
    task_mapping: Arc<RwLock<TaskStore<Arc<TaskInQueue>>>>,
    task_priority: Arc<RwLock<HashMap<TaskId, OrderedTaskPriority>>>,
    semaphore: Arc<Semaphore>,
    notifier: Arc<Notify>,
    tx: Sender<TaskPayload>,
    // after the tasks in subscriptions finished, the task itself continue to run
    task_subscription: Arc<RwLock<HashMap<TaskId, Vec<TaskId>>>>,
    // after task finished, trigger the tasks in dispatch
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
        let task_mapping = Arc::new(RwLock::new(TaskStore::new()));

        let cpu_task_ctx = TaskPoolContext {
            task_bound: TaskBound::CPU,
            task_queue: Arc::new(RwLock::new(PriorityQueue::new())),
            task_mapping: task_mapping.clone(),
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
            task_mapping: task_mapping.clone(),
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
                match payload {
                    TaskPayload::Task(NewTaskPayload {
                        file_identifier,
                        file_path,
                        task_type,
                        priority,
                        notifier,
                    }) => {
                        let task_type = task_type.clone();

                        let task_id = TaskId::new(&file_identifier, &task_type);
                        tracing::info!("Task received: {}", &task_id);

                        // if task exists, ignore it
                        // 这里 task_queue 是所有未执行的任务
                        // task_mapping 还包括了正在执行的任务，所以用它
                        if let Some(_) = task_mapping.read().await.get(&task_id.to_store_key()) {
                            continue;
                        }

                        let task = Task {
                            file_identifier: file_identifier.clone(),
                            file_path: file_path.clone(),
                            task_type: task_type.clone(),
                        };
                        let task = Arc::new(task);

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

                        let task_bound = get_task_bound(&task_type).await;
                        let task_ctx = match task_bound {
                            TaskBound::CPU => &cpu_ctx_clone,
                            TaskBound::IO => &io_ctx_clone,
                        };

                        let current_cancel_token = CancellationToken::new();
                        let priority: OrderedTaskPriority = priority.into();
                        // 通过 order 确保在相同优先级和时间戳的情况下，先加入的任务优先级相对更高
                        let priority = {
                            let task_queue = task_ctx.task_queue.read().await;
                            priority.with_insert_order(task_queue.len())
                        };

                        let task_in_queue = TaskInQueue {
                            task: task.clone(),
                            priority: priority.into(),
                            cancel_token: current_cancel_token.clone(),
                            notifier,
                        };

                        // record task in mapping
                        {
                            let mut task_mapping = task_mapping.write().await;
                            task_mapping.set(&task_id.to_store_key(), Arc::new(task_in_queue));
                        }

                        {
                            let mut task_queue = task_ctx.task_queue.write().await;
                            task_queue.push(task_id, priority);

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
                                        task_mapping.get_mut(&task_id.to_store_key())
                                    {
                                        item.cancel().await;

                                        // 需要把任务重新添加回去
                                        let mut task_queue = task_ctx.task_queue.write().await;
                                        task_queue.push(task_id.clone(), item.priority.clone());
                                    }
                                }
                            }
                        }

                        task_ctx.notifier.notify_one();
                    }
                    TaskPayload::CancelByIdAndType(file_identifier, task_type) => {
                        let task_mapping = task_mapping.read().await;
                        let task_id = TaskId::new(&file_identifier, &task_type);
                        if let Some(task) = task_mapping.get(&task_id.to_store_key()) {
                            task.cancel().await;
                        }
                    }
                    TaskPayload::CancelById(file_identifier) => {
                        let task_mapping = task_mapping.read().await;
                        for (_, task) in task_mapping.get_all(&format!("{}*", file_identifier)) {
                            task.cancel().await;
                        }
                    }
                    TaskPayload::CancelAll => {
                        let task_mapping = task_mapping.read().await;
                        for (_, task) in task_mapping.get_all("*") {
                            task.cancel().await;
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

            if let Some((task_id, priority)) = task {
                let semaphore = self.semaphore.clone();

                let current_task = match self.task_mapping.read().await.get(&task_id.to_store_key())
                {
                    Some(current_task) => current_task.clone(),
                    _ => {
                        tracing::error!("task not found: {}", &task_id);
                        drop(permit);
                        continue;
                    }
                };

                tracing::debug!("Task popped: {}", &task_id);

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
                                .map(|v| TaskId::new(task_id.file_identifier(), v))
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

                                // create new dependent tasks
                                let mut payload = NewTaskPayload::new(
                                    dep.file_identifier(),
                                    &current_task.task.file_path,
                                    dep.task_type(),
                                );
                                payload.with_priority(Some(current_task.priority.into()));
                                payload.with_notifier(current_task.notifier.clone());

                                if let Err(e) = self.tx.send(TaskPayload::Task(payload)).await {
                                    tracing::error!("Failed to add dependent task: {}", e);
                                }
                            }

                            // 不执行任务了
                            drop(permit);

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
                let task_queue = self.task_queue.clone();

                tokio::spawn(async move {
                    {
                        let mut task_priority = task_priority.write().await;
                        task_priority.insert(task_id.clone(), priority);
                    }

                    let task_id = current_task.task.id();

                    let file_info = FileInfo {
                        file_identifier: task_id.file_identifier().to_string(),
                        file_path: current_task.task.file_path.clone(),
                    };

                    if let Some(tx) = current_task.notifier.clone() {
                        if let Err(_) = tx
                            .send(TaskNotification::new(
                                task_id.task_type(),
                                TaskStatus::Started,
                                None,
                            ))
                            .await
                        {
                            tracing::error!("Failed to send task start notification");
                        }
                    }

                    tokio::select! {
                        result = current_task.task.task_type.run(&file_info, &content_base) => {
                            let notification = match &result {
                                Err(e) => {
                                    tracing::error!("Task error: {} {}", &task_id, e);
                                    TaskNotification::new(
                                        task_id.task_type(),
                                        TaskStatus::Error,
                                        Some(e.to_string().as_str()),
                                    )
                                }
                                _ => {
                                    tracing::info!("Task finished: {}", &task_id);
                                    TaskNotification::new(
                                        task_id.task_type(),
                                        TaskStatus::Finished,
                                        None
                                    )
                                }
                            };

                            if let Some(tx) = current_task.notifier.clone() {
                                if let Err(_) = tx.send(notification).await {
                                    tracing::error!("Failed to send task result");
                                }
                            }
                        }
                        _ = current_task.cancel_token.cancelled() => {
                            if let Some(tx) = &current_task.notifier {
                                if let Err(e) = tx
                                    .send(TaskNotification {
                                        task_type: current_task.task.task_type.clone(),
                                        status: TaskStatus::Cancelled,
                                        message: None,
                                    })
                                    .await
                                {
                                    tracing::error!("Failed to send task cancelled notification: {}", e);
                                }
                            }

                            tracing::info!("Spawned task has been cancelled: {}", &task_id);
                        }
                    }

                    // add permit back
                    semaphore.add_permits(1);

                    {
                        let mut task_mapping = task_mapping.write().await;
                        task_mapping.remove(&task_id.to_store_key());
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
                        let mut task_queue = task_queue.write().await;
                        let task_mapping = task_mapping.read().await;

                        if let Some(targets) = task_dispatch.remove(&task_id) {
                            // targets are the tasks that should be awaked
                            for target in targets.iter() {
                                match task_subscription.get_mut(target) {
                                    Some(v) => {
                                        v.retain(|x| x != &task_id);

                                        // if subscription is empty, the target task can be executed safely
                                        if v.is_empty() {
                                            if let Some(task) =
                                                task_mapping.get(&target.to_store_key())
                                            {
                                                task_queue.push(task.task.id(), task.priority);
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
