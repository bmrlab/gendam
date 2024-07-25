use crate::{
    record::{TaskRunOutput, TaskRunRecord},
    ContentBase, ContentTaskType,
};
use async_trait::async_trait;
use serde_json::Value;
use std::path::PathBuf;
use storage::Storage;

#[derive(Clone, Debug)]
pub struct FileInfo {
    pub file_identifier: String,
    pub file_path: PathBuf,
}

#[async_trait]
pub trait ContentTask: Into<ContentTaskType> + Clone + Storage {
    /// Task entrypoint. This should not be overridden.
    async fn run(&self, file_info: &FileInfo, ctx: &ContentBase) -> anyhow::Result<()> {
        let task_type = self.clone().into();

        // TODO check if the task is already exist
        let mut task_run_record = ctx
            .create_task(&file_info.file_identifier, &task_type)
            .await?;

        self.inner_run(file_info, ctx, &mut task_run_record).await?;

        ctx.set_task_run_record(&file_info.file_identifier, &task_type, &task_run_record)
            .await?;
        Ok(())
    }

    /// Task implementation. Every task should implement its own `inner_run`.
    async fn inner_run(
        &self,
        file_info: &FileInfo,
        ctx: &ContentBase,
        task_run_record: &mut TaskRunRecord,
    ) -> anyhow::Result<()>;

    /// Get task output path from the latest `TaskRunRecord`.
    async fn task_output_path(
        &self,
        file_info: &crate::FileInfo,
        ctx: &crate::ContentBase,
    ) -> anyhow::Result<PathBuf> {
        let task_run_record = ctx
            .task_run_record(&file_info.file_identifier, &self.clone().into())
            .await?;
        let task_output = self.task_output(&task_run_record).await?;
        let output_path = task_output
            .to_path_buf(&file_info.file_identifier, ctx)
            .await?;
        Ok(output_path)
    }

    fn task_parameters(&self, ctx: &ContentBase) -> Value;
    async fn task_output(&self, task_run_record: &TaskRunRecord) -> anyhow::Result<TaskRunOutput>;

    /// The direct dependencies of the task. Do not include the task itself and the dependencies of the dependencies.
    fn task_dependencies(&self) -> Vec<ContentTaskType>;
}
