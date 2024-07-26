use std::path::PathBuf;

use crate::priority::TaskPriority;
use content_base_task::ContentTaskType;


#[derive(Hash, PartialEq, Eq, Clone, Debug)]
pub(crate) struct Task {
    pub file_identifier: String,
    pub file_path: PathBuf,
    pub task_type: ContentTaskType,
}

pub enum TaskPayload {
    Task(String, PathBuf, ContentTaskType, TaskPriority),
    CancelByIdAndType(String, ContentTaskType),
    CancelById(String),
    CancelAll,
}
