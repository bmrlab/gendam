use content_base_task::ContentTaskType;

#[derive(Clone, Debug)]
pub enum TaskStatus {
    Init,
    Started,
    Finished,
    Error,
    Cancelled,
}

#[derive(Debug, Clone)]
pub struct TaskNotification {
    pub task_type: ContentTaskType,
    pub status: TaskStatus,
    pub message: Option<String>,
}
