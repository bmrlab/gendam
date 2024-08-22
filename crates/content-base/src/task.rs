use crate::ContentBase;
use content_base_task::ContentTaskType;

pub struct CancelTaskPayload {
    file_identifier: String,
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

pub struct ArtifactPayload {}

impl ContentBase {
    pub async fn cancel_task(&self, payload: CancelTaskPayload) {
        todo!()
    }
}
