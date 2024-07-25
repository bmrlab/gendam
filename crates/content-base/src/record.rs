use crate::{metadata::ContentMetadata, ContentBase, ContentTask, ContentTaskType};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::HashMap, path::PathBuf};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct TaskRunDependency {
    task_type: ContentTaskType,
    run_id: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub enum TaskRunOutput {
    Data(Value),
    File(PathBuf),
    Folder(PathBuf),
}

impl TaskRunOutput {
    pub async fn to_path_buf(&self, file_identifier: &str, ctx: &ContentBase) -> anyhow::Result<PathBuf> {
        let artifacts_dir = ctx.artifacts_dir(file_identifier);
        let path = match self {
            Self::File(path) => Some(path.clone()),
            Self::Folder(path) => Some(path.clone()),
            _ => None,
        };
        path.map(|v| artifacts_dir.join(v)).ok_or(anyhow::anyhow!("do not contain output path"))
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct TaskRunRecord {
    id: String,
    task_type: ContentTaskType,
    completed: bool,
    parameters: Option<Value>,
    output: Option<TaskRunOutput>,
    /// If dependencies length is 0, the task is not dependent on other tasks.
    dependencies: Vec<TaskRunDependency>,
}

impl TaskRunRecord {
    pub fn new(task_type: &ContentTaskType) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            task_type: task_type.clone(),
            completed: false,
            parameters: None,
            output: None,
            dependencies: vec![],
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn is_completed(&self) -> bool {
        self.completed
    }

    pub fn output(&self) -> Option<&TaskRunOutput> {
        self.output.as_ref()
    }

    pub fn complete(&mut self) {
        self.completed = true;
    }

    pub fn with_deps(&mut self, deps: &[TaskRunDependency]) {
        self.dependencies = deps.into();
    }

    pub fn with_parameters(&mut self, parameters: &Value) {
        self.parameters = Some(parameters.clone());
    }

    pub fn with_output(&mut self, output: &TaskRunOutput) {
        self.output = Some(output.clone());
    }

    pub async fn output_path(&self, file_identifier: &str, ctx: &ContentBase) -> anyhow::Result<PathBuf> {
        let output  = self.task_type.task_output(self).await?;
        output.to_path_buf(file_identifier, ctx).await
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct TaskRecord {
    file_identifier: String,
    metadata: ContentMetadata,
    tasks: HashMap<ContentTaskType, Vec<TaskRunRecord>>,
}

impl TaskRecord {
    pub fn new(file_identifier: &str, metadata: Option<ContentMetadata>) -> Self {
        Self {
            file_identifier: file_identifier.to_string(),
            metadata: metadata.unwrap_or_default(),
            tasks: HashMap::new(),
        }
    }

    pub fn with_metadata(mut self, metadata: &ContentMetadata) -> Self {
        self.metadata = metadata.clone();
        self
    }

    pub fn with_tasks(mut self, tasks: &HashMap<ContentTaskType, Vec<TaskRunRecord>>) -> Self {
        self.tasks = tasks.clone();
        self
    }

    pub fn metadata(&self) -> &ContentMetadata {
        &self.metadata
    }

    pub fn tasks(&self) -> &HashMap<ContentTaskType, Vec<TaskRunRecord>> {
        &self.tasks
    }
}
