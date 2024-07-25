use crate::{task::ContentTaskType, ContentTask};
use content_base_context::ContentBaseCtx;
use content_metadata::ContentMetadata;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};
use storage_macro::Storage;

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
    pub async fn to_path_buf(
        &self,
        file_identifier: &str,
        ctx: &ContentBaseCtx,
    ) -> anyhow::Result<PathBuf> {
        let artifacts_dir = ctx.artifacts_dir(file_identifier);
        let path = match self {
            Self::File(path) => Some(path.clone()),
            Self::Folder(path) => Some(path.clone()),
            _ => None,
        };
        path.map(|v| artifacts_dir.join(v))
            .ok_or(anyhow::anyhow!("do not contain output path"))
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

    pub async fn output_path(
        &self,
        file_identifier: &str,
        ctx: &ContentBaseCtx,
    ) -> anyhow::Result<PathBuf> {
        let output = self.task_type.task_output(self).await?;
        output.to_path_buf(file_identifier, ctx).await
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, Storage)]
pub struct TaskRecord {
    file_identifier: String,
    metadata: ContentMetadata,
    tasks: HashMap<ContentTaskType, Vec<TaskRunRecord>>,
}

impl TaskRecord {
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

    pub fn path(file_identifier: &str, ctx: &ContentBaseCtx) -> impl AsRef<Path> {
        ctx.artifacts_dir(file_identifier).join("artifacts.json")
    }

    async fn save(&self, ctx: &ContentBaseCtx) -> anyhow::Result<()> {
        self.write(
            Self::path(&self.file_identifier, ctx).as_ref().to_path_buf(),
            serde_json::to_string(self)?.into(),
        )
        .await?;
        Ok(())
    }

    pub async fn from_content_base(file_identifier: &str, ctx: &ContentBaseCtx) -> Self {
        // FIXME self.read_to_string can only be called by object not but Self::
        // so create a fake self to call read_to_string
        let fake_self = Self {
            file_identifier: file_identifier.to_string(),
            metadata: ContentMetadata::default(),
            tasks: HashMap::new(),
        };

        match fake_self.read_to_string(
            Self::path(file_identifier, ctx).as_ref().to_path_buf()
        ) {
            Ok(record) => match serde_json::from_str::<TaskRecord>(&record) {
                Ok(record) => record,
                _ => fake_self
            }
            _ => fake_self
        }
    }

    pub async fn add_task_run(&mut self, ctx: &ContentBaseCtx, task_type: &ContentTaskType) -> anyhow::Result<TaskRunRecord> {
        let mut tasks = self.tasks.clone();
        if !tasks.contains_key(task_type) {
            tasks.insert(task_type.clone(), vec![]);
        }

        let mut task_run_record = TaskRunRecord::new(task_type);

        // FIXME Add task dependencies according to map
        // task_run_record.with_deps()
        let task_parameters = task_type.task_parameters(ctx);
        let task_output = task_type.task_output(&task_run_record).await?;
        task_run_record.with_output(&task_output);
        task_run_record.with_parameters(&task_parameters);

        let runs = tasks.get_mut(task_type).expect("data has been inserted");
        runs.push(task_run_record.clone());

        self.tasks = tasks;
        self.save(ctx).await?;

        Ok(task_run_record)
    }

    pub async fn update_task_run(&mut self, ctx: &ContentBaseCtx, task_run_record: &TaskRunRecord) -> anyhow::Result<()> {
        let mut tasks = self.tasks().clone();
        let runs = tasks
            .get_mut(&task_run_record.task_type)
            .ok_or(anyhow::anyhow!("task record not found"))?;
        let index = runs
            .iter()
            .position(|task| task.id() == task_run_record.id())
            .ok_or(anyhow::anyhow!("task run record not found"))?;
        runs[index] = task_run_record.clone();
        self.tasks = tasks;

        self.save(ctx).await?;

        Ok(())
    }

    pub async fn latest_run(file_identifier: &str, ctx: &ContentBaseCtx, task_type: &ContentTaskType) -> anyhow::Result<TaskRunRecord> {
        let record = Self::from_content_base(file_identifier, ctx).await;
        record
            .tasks()
            .get(task_type)
            .and_then(|v| v.last())
            .cloned()
            .ok_or(anyhow::anyhow!("task run record not found"))
    }

    pub async fn set_metadata(&mut self, ctx: &ContentBaseCtx, metadata: &ContentMetadata) -> anyhow::Result<()> {
        self.metadata = metadata.clone();
        self.save(ctx).await
    }
}
