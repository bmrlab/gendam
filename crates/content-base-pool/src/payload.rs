use crate::{priority::TaskPriority, TaskNotification};
use content_base_task::ContentTaskType;
use std::path::{Path, PathBuf};
use tokio::sync::mpsc;

#[derive(Hash, PartialEq, Eq, Clone, Debug)]
pub(crate) struct Task {
    pub file_identifier: String,
    pub file_path: PathBuf,
    pub task_type: ContentTaskType,
}

pub struct NewTaskPayload {
    pub file_identifier: String,
    pub file_path: PathBuf,
    pub task_type: ContentTaskType,
    pub priority: TaskPriority,
    pub notifier: Option<mpsc::Sender<TaskNotification>>,
}

impl NewTaskPayload {
    pub fn new(
        file_identifier: &str,
        file_path: impl AsRef<Path>,
        task_type: impl Into<ContentTaskType>,
    ) -> Self {
        Self {
            file_identifier: file_identifier.to_string(),
            file_path: file_path.as_ref().to_path_buf(),
            task_type: task_type.into(),
            priority: TaskPriority::Normal,
            notifier: None,
        }
    }

    pub fn with_priority(&mut self, priority: Option<TaskPriority>) {
        self.priority = priority.unwrap_or(TaskPriority::Normal);
    }

    pub fn with_notifier(&mut self, notifier: Option<mpsc::Sender<TaskNotification>>) {
        self.notifier = notifier;
    }
}

pub enum TaskPayload {
    Task(NewTaskPayload),
    CancelByIdAndType(String, ContentTaskType),
    CancelById(String),
    CancelAll,
}
