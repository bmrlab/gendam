use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use strum_macros::AsRefStr;

#[derive(AsRefStr, Clone, Copy, strum_macros::Display, Debug)]
pub enum TaskPriority {
    #[strum(serialize = "0")]
    Low,
    #[strum(serialize = "5")]
    Normal,
    #[strum(serialize = "10")]
    High,
}

pub trait FileHandlerTaskType: Copy + Display + Send + Sync {
    fn try_from_any_task_type(task_type: impl FileHandlerTaskType) -> anyhow::Result<Self>;
    fn priority(&self) -> TaskPriority;
}

pub trait FileMetadata: Clone + Serialize + for<'a> Deserialize<'a> {}

#[async_trait]
pub trait FileHandler: Send + Sync {
    async fn run_task(&self, task_type: &str) -> anyhow::Result<()>;
    async fn delete_task_artifacts(&self, task_type: &str) -> anyhow::Result<()>;
    async fn update_database(&self) -> anyhow::Result<()>;

    fn get_supported_task_types(&self) -> Vec<(String, TaskPriority)>;
    // fn metadata(&self) -> anyhow::Result<impl FileMetadata>;
}

impl From<TaskPriority> for usize {
    fn from(priority: TaskPriority) -> Self {
        priority.to_string().parse().unwrap()
    }
}

impl PartialEq for TaskPriority {
    fn eq(&self, other: &Self) -> bool {
        usize::from(*self) == usize::from(*other)
    }
}

impl Eq for TaskPriority {}

impl PartialOrd for TaskPriority {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TaskPriority {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        usize::from(*self).cmp(&usize::from(*other))
    }
}
