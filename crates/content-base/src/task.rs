use crate::ContentBase;
use content_base_task::ContentTaskType;

pub struct CancelTaskPayload {
    file_identifier: String,
    /// Which tasks to cancel, None means cancel all tasks.
    tasks: Option<Vec<ContentTaskType>>,
}

impl CancelTaskPayload {
    pub fn new(file_identifier: &str) -> Self {
        Self {
            file_identifier: file_identifier.to_string(),
            tasks: None,
        }
    }

    pub fn with_tasks(mut self, tasks: &[ContentTaskType]) -> Self {
        self.tasks = Some(tasks.iter().map(|v| v.clone()).collect::<Vec<_>>());
        self
    }
}

impl ContentBase {
    pub async fn cancel_task(&self, payload: CancelTaskPayload) -> anyhow::Result<()> {
        match payload.tasks {
            Some(tasks) => {
                for task in tasks {
                    self.task_pool
                        .cancel_specific(&payload.file_identifier, &task)
                        .await?;
                }
            }
            _ => {
                self.task_pool
                    .cancel_by_file(&payload.file_identifier)
                    .await?;
            }
        }

        Ok(())
    }
}
