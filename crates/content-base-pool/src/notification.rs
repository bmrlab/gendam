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

impl TaskNotification {
    pub fn new(task_type: &ContentTaskType, status: TaskStatus, message: Option<&str>) -> Self {
        Self {
            task_type: task_type.clone(),
            status,
            message: message.map(|s| s.to_string()),
        }
    }
}
