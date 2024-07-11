use async_trait::async_trait;
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

#[async_trait]
pub trait FileHandler: Send + Sync {
    async fn run_task(
        &self,
        task_type: &str,
        with_existing_artifacts: Option<bool>,
    ) -> anyhow::Result<()>;
    async fn delete_artifacts_in_db(&self) -> anyhow::Result<()>;
    async fn delete_artifacts(&self) -> anyhow::Result<()>;
    async fn delete_artifacts_in_db_by_task(&self, task_type: &str) -> anyhow::Result<()>;
    async fn delete_artifacts_by_task(&self, task_type: &str) -> anyhow::Result<()>;

    fn get_supported_task_types(&self) -> Vec<(String, TaskPriority)>;
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
