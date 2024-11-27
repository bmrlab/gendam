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
use tracing::Instrument;

#[derive(Clone, Debug)]
pub struct TaskPool {
    tx: mpsc::Sender<TaskPayload>,
}

#[derive(Clone)]
struct TaskInQueue {
    task: Arc<Task>,
    priority: OrderedTaskPriority,
    /// 仅用于高优先级任务取消低优先级任务，因为是临时取消，不会清空 task_mapping 中的任务。
    /// 如果要永久取消（比如人工取消）的功能，需要用另一个 drop_cancel_token 来做。
    priority_cancel_token: CancellationToken,
    /// 用于永久取消任务，也就是丢弃任务，这个会在素材被删除的时候用到。
    drop_cancel_token: CancellationToken,
    notifier: Option<mpsc::Sender<TaskNotification>>,
}

impl TaskInQueue {
    async fn cancel_due_to_priority(&self) {
        self.priority_cancel_token.cancel();
        tracing::info!(
            file_identifier=%self.task.file_identifier,
            task_type=%self.task.task_type,
            "Task cancelled due to priority",
        );
    }
    async fn drop_with_reason(&self, drop_reason: &str) {
        self.drop_cancel_token.cancel();
        tracing::info!(
            file_identifier=%self.task.file_identifier,
            task_type=%self.task.task_type,
            reason=%drop_reason,
            "Task dropped",
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
        // 这里是从队列里 pop 出来下一个要执行的任务，丢入 queue 里
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
                            file_full_path_on_disk: file_path.clone(),
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

                        let priority: OrderedTaskPriority = priority.into();
                        // 通过 order 确保在相同优先级和时间戳的情况下，先加入的任务优先级相对更高
                        let priority = {
                            let task_queue = task_ctx.task_queue.read().await;
                            priority.with_insert_order(task_queue.len())
                        };

                        let task_in_queue = TaskInQueue {
                            task: task.clone(),
                            priority: priority.into(),
                            priority_cancel_token: CancellationToken::new(),
                            drop_cancel_token: CancellationToken::new(),
                            notifier,
                        };

                        // record task in mapping
                        {
                            let mut task_mapping = task_mapping.write().await;
                            task_mapping.set(&task_id.to_store_key(), Arc::new(task_in_queue));
                        }

                        {
                            let mut task_queue = task_ctx.task_queue.write().await;
                            tracing::debug!(task_id=%task_id, priority=%priority, "task_queue lock acquired to push task");
                            task_queue.push(task_id.clone(), priority);
                            // 释放 task_queue 的锁，不然下面把任务重新添加回去需要 task_queue.write() 的时候会死锁
                        }

                        {
                            // 处理任务优先级
                            let current_priority = priority;
                            if task_ctx.semaphore.available_permits() == 0 {
                                // 找到优先级低于当前任务的任务，并取消它
                                let mut task_priority = task_ctx.task_priority.write().await;
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

                                if let Some(min_task_id) = min_task_id {
                                    task_priority.remove(&min_task_id);
                                    drop(task_priority); // drop 防止死锁的可能性, 并确保接下来不再使用它

                                    tracing::info!(
                                        "Task {} will be canceled due to a higher priority task {}",
                                        &min_task_id,
                                        &task_id
                                    );
                                    drop(task_id); // 确保后面不再使用了
                                    let mut task_mapping = task_ctx.task_mapping.write().await;
                                    if let Some(task_in_queue) =
                                        task_mapping.get_mut(&min_task_id.to_store_key())
                                    {
                                        // cancel 的流程
                                        // -> cancel_token.cancel()
                                        // -> tokio::select! { _ = current_task.cancel_token.cancelled() }
                                        // -> semaphore.add_permits(1)
                                        // -> pop 下一个任务，更高优先级的任务
                                        task_in_queue.cancel_due_to_priority().await;

                                        // 需要把任务重新添加回去
                                        let priority = task_in_queue.priority;
                                        let mut task_queue = task_ctx.task_queue.write().await;
                                        tracing::debug!(task_id=%min_task_id, priority=%priority, "task_queue lock acquired to push back canceled task");
                                        task_queue.push(min_task_id, priority);
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
                            task.drop_with_reason("CancelByIdAndType received").await;
                        }
                    }
                    TaskPayload::CancelById(file_identifier) => {
                        let task_mapping = task_mapping.read().await;
                        for (_, task) in task_mapping.get_all(&format!("{}*", file_identifier)) {
                            task.drop_with_reason("CancelById received").await;
                        }
                    }
                    TaskPayload::CancelAll => {
                        let task_mapping = task_mapping.read().await;
                        for (_, task) in task_mapping.get_all("*") {
                            task.drop_with_reason("CancelAll received").await;
                        }
                    }
                }
            }
        });

        // loop for task execution
        let cb = content_base.clone();
        tokio::spawn(async move {
            cpu_task_ctx.loop_for_task_execution(&cb).await;
        });

        let cb = content_base.clone();
        tokio::spawn(async move {
            io_task_ctx.loop_for_task_execution(&cb).await;
        });

        Ok(Self { tx })
    }

    pub async fn add_task(
        &self,
        file_identifier: &str,
        file_path: impl AsRef<Path>,
        task_type: impl Into<ContentTaskType>,
        priority: Option<TaskPriority>,
        notifier: Option<mpsc::Sender<TaskNotification>>,
    ) -> anyhow::Result<()> {
        let mut payload = NewTaskPayload::new(file_identifier, file_path, task_type.into());
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
    /// 为了更好的 tracing 把方法分成 pop_next_task 和 async_exec_task
    /// 这是个无限循环，需要每次 pop 新任务的时候创建一个 span
    /// 但 run 是个 async 方法，async 代码中的 span 需要额外小心，最好都用 tracing::instrument
    /// https://docs.rs/tracing/latest/tracing/struct.Span.html#in-asynchronous-code
    ///
    /// 如果是 run 方法里面直接用 tracing::info_span!，需要创建一个 span 然后立即 enter，
    /// 接下来用 `async {}.instrument(span)` 和 `span.in_scope(|| {})` 来执行 async 和 sync 的代码，有点麻烦
    pub async fn loop_for_task_execution(&self, content_base: &ContentBaseCtx) {
        let mut count: usize = 0;
        let mut task_interval = tokio::time::interval(std::time::Duration::from_secs(1));
        let mut status_interval = tokio::time::interval(std::time::Duration::from_secs(60));
        loop {
            tokio::select! {
                // 任务执行
                _ = task_interval.tick() => {
                    let Some((task_id, priority, current_task)) = self.pop_next_task(count + 1).await else {
                        continue
                    };
                    self.async_exec_task(content_base, task_id, priority, current_task).await;
                    count += 1;
                }
                // 状态打印
                _ = status_interval.tick() => {
                    let len = self.task_queue.read().await.len();
                    let bound = match self.task_bound {
                        TaskBound::CPU => "CPU",
                        TaskBound::IO => "IO",
                    };
                    tracing::info!(queue=%bound, length=%len, processed=%count, "loop_for_task_execution");
                }
            }
        }
    }

    #[tracing::instrument(level = "info", skip(self))]
    async fn pop_next_task(
        &self,
        _count: usize, // 仅用于 tracing
    ) -> Option<(TaskId, OrderedTaskPriority, Arc<TaskInQueue>)> {
        let permit = match self.semaphore.acquire().await {
            Ok(permit) => permit,
            Err(e) => {
                tracing::error!(error = ?e, "semaphore acquire failed");
                return None;
            }
        };

        // 这里是执行下一个任务的入口，从 queue 里取出来
        let (task_id, priority) = match {
            let task = self.task_queue.write().await.pop();
            task
        } {
            Some((task_id, priority)) => (task_id, priority),
            None => {
                // tracing::warn!("task queue is empty");  // 会一直输出，因为是个 loop
                drop(permit);
                return None;
            }
        };

        let current_task = match self.task_mapping.read().await.get(&task_id.to_store_key()) {
            Some(current_task) => current_task.clone(),
            _ => {
                tracing::error!("task not found: {}", &task_id);
                drop(permit);
                return None;
            }
        };

        tracing::info!("Task popped");

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
                            &current_task.task.file_full_path_on_disk,
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

                    return None;
                }
            }
        }

        // Here we can just forget the permit, and increase permit after task is finished
        // by calling `add_permits` on semaphore.
        permit.forget();

        Some((task_id, priority, current_task))
    }

    #[tracing::instrument(skip_all, fields(hash = %task_id.file_identifier(), task_type = %task_id.task_type()))]
    async fn async_exec_task(
        &self,
        content_base: &ContentBaseCtx,
        task_id: TaskId,
        priority: OrderedTaskPriority,
        current_task: Arc<TaskInQueue>,
    ) {
        let content_base = content_base.clone();
        let semaphore = self.semaphore.clone();
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
                file_full_path_on_disk: current_task.task.file_full_path_on_disk.clone(),
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

            tracing::info!("Task started");
            tokio::select! {
                // 真的开始执行一个任务了
                result = current_task.task.task_type.run(&file_info, &content_base) => {
                    let notification = match &result {
                        Err(e) => {
                            tracing::error!(task_id=%task_id, "Task error: {}", e);
                            TaskNotification::new(
                                task_id.task_type(),
                                TaskStatus::Error,
                                Some(e.to_string().as_str()),
                            )
                        }
                        _ => {
                            tracing::info!(task_id=%task_id, "Task finished");
                            TaskNotification::new(
                                task_id.task_type(),
                                TaskStatus::Finished,
                                None
                            )
                        }
                    };

                    if let Some(tx) = current_task.notifier.clone() {
                        if let Err(_) = tx.send(notification).await {
                            tracing::error!(task_id=%task_id, "Failed to send task result");
                        }
                    }

                    // remove task from task_mapping
                    task_mapping.write().await.remove(&task_id.to_store_key());
                    tracing::debug!(task_id=%task_id, "Remove task from task_mapping");
                }
                _ = current_task.priority_cancel_token.cancelled() => {
                    tracing::info!(task_id=%task_id, "Spawned task has been cancelled due to priority");
                    if let Some(tx) = &current_task.notifier {
                        if let Err(e) = tx
                            .send(TaskNotification::new(
                                &current_task.task.task_type,
                                TaskStatus::Cancelled,
                                Some("Task cancelled due to priority")
                            ))
                            .await
                        {
                            tracing::error!(task_id=%task_id, error=?e, "Failed to send task cancelled notification");
                        }
                    }
                    // keep task in task_mapping since it will be popped out again, but with new cancel tokens
                    let mut task_mapping = task_mapping.write().await;
                    if let Some(removed) = task_mapping.remove(&task_id.to_store_key()) {
                        let removed = removed.as_ref().to_owned();
                        let task_in_queue = TaskInQueue {
                            task: removed.task,
                            priority: removed.priority,
                            priority_cancel_token: CancellationToken::new(),
                            drop_cancel_token: CancellationToken::new(),
                            notifier: removed.notifier,
                        };
                        task_mapping.set(&task_id.to_store_key(), Arc::new(task_in_queue));
                    }
                    tracing::debug!("Keep task in task_mapping with new cancel tokens");
                }
                _ = current_task.drop_cancel_token.cancelled() => {
                    tracing::info!(task_id=%task_id, "Spawned task has been dropped");
                    if let Some(tx) = &current_task.notifier {
                        if let Err(e) = tx
                            .send(TaskNotification::new(
                                &current_task.task.task_type,
                                TaskStatus::Cancelled,
                                Some("Task dropped"),
                            ))
                            .await
                        {
                            tracing::error!(task_id=%task_id, error=?e, "Failed to send task cancelled notification");
                        }
                    }
                    // remove task from task_mapping
                    task_mapping.write().await.remove(&task_id.to_store_key());
                    tracing::debug!(task_id=%task_id, "Remove task from task_mapping");
                }
            }

            // add permit back
            semaphore.add_permits(1);
            tracing::info!("semaphore permits is added back");

            // 移动到了 tokio::select! 里面根据不同情况决定是否要删除
            // {
            //     let mut task_mapping = task_mapping.write().await;
            //     task_mapping.remove(&task_id.to_store_key());
            //     tracing::debug!(task_id=%task_id, "Remove task from task_mapping");
            // }

            // 但是 task_priority 照常处理，见 https://github.com/bmrlab/gendam/issues/109#issuecomment-2505306961
            {
                let mut task_priority = task_priority.write().await;
                task_priority.remove(&task_id);
                tracing::debug!(task_id=%task_id, "Remove task from task_priority");
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
                                    if let Some(task) = task_mapping.get(&target.to_store_key()) {
                                        task_queue.push(task.task.id(), task.priority);
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }.instrument(tracing::Span::current()));
        // 把 span 信息带到 acync block 里
    }
}
