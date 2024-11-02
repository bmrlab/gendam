use crate::{priority::TaskPriority, TaskNotification};
use content_base_task::ContentTaskType;
use std::path::{Path, PathBuf};
use tokio::sync::mpsc;

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct TaskId {
    file_identifier: String,
    task_type: ContentTaskType,
}

impl std::fmt::Display for TaskId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.file_identifier, self.task_type)
    }
}

impl TaskId {
    pub fn new(file_identifier: &str, task_type: &ContentTaskType) -> Self {
        Self {
            file_identifier: file_identifier.to_string(),
            task_type: task_type.clone(),
        }
    }

    pub fn to_store_key(&self) -> String {
        format!("{}:{}", self.file_identifier, self.task_type)
    }

    pub fn file_identifier(&self) -> &str {
        &self.file_identifier
    }

    pub fn task_type(&self) -> &ContentTaskType {
        &self.task_type
    }
}

#[derive(Hash, PartialEq, Eq, Clone, Debug)]
pub(crate) struct Task {
    pub file_identifier: String,
    pub file_full_path_on_disk: PathBuf,
    pub task_type: ContentTaskType,
}

impl Task {
    pub fn id(&self) -> TaskId {
        TaskId::new(&self.file_identifier, &self.task_type)
    }
}

pub struct NewTaskPayload {
    pub file_identifier: String,
    /// Full path to the file on disk
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
