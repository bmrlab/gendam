use crate::{ContentTaskType, FileInfo, TaskRecord, TaskRunOutput, TaskRunRecord};
use async_trait::async_trait;
use content_base_context::ContentBaseCtx;
use serde_json::Value;
use std::path::PathBuf;
use storage::Storage;

#[async_trait]
pub trait ContentTask: Into<ContentTaskType> + Clone + Storage {
    /// Task entrypoint. This should not be overridden.
    async fn run(&self, file_info: &FileInfo, ctx: &ContentBaseCtx) -> anyhow::Result<()> {
        let task_type = self.clone().into();

        let mut task_record = TaskRecord::from_content_base(&file_info.file_identifier, ctx).await;

        if let Some(target_task_run) = task_record.target_run(ctx, &task_type) {
            if target_task_run.is_completed() {
                tracing::info!("Completed task run found, finish task.");

                return Ok(());
            }
        }

        let mut task_run_record = task_record.add_task_run(ctx, &task_type).await?;

        self.inner_run(file_info, ctx, &mut task_run_record).await?;

        task_run_record.complete();
        task_run_record.update_deps(ctx, &task_record)?;
        task_record.update_task_run(ctx, &task_run_record).await?;

        Ok(())
    }

    /// Task implementation. Every task should implement its own `inner_run`.
    async fn inner_run(
        &self,
        file_info: &FileInfo,
        ctx: &ContentBaseCtx,
        task_run_record: &mut TaskRunRecord,
    ) -> anyhow::Result<()>;

    /// Get task output path from the target `TaskRunRecord`.
    async fn task_output_path(
        &self,
        file_info: &FileInfo,
        ctx: &ContentBaseCtx,
    ) -> anyhow::Result<PathBuf> {
        let task_record = TaskRecord::from_content_base(&file_info.file_identifier, ctx).await;
        let task_run_record = task_record
            .target_run(ctx, &self.clone().into())
            .ok_or(anyhow::anyhow!("no target run found"))?;
        let task_output = self.task_output(&task_run_record).await?;
        let output_path = task_output
            .to_path_buf(&file_info.file_identifier, ctx)
            .await?;
        Ok(output_path)
    }

    fn task_parameters(&self, ctx: &ContentBaseCtx) -> Value;
    async fn task_output(&self, task_run_record: &TaskRunRecord) -> anyhow::Result<TaskRunOutput>;

    /// The direct dependencies of the task. Do not include the task itself and the dependencies of the dependencies.
    fn task_dependencies(&self) -> Vec<ContentTaskType>;
}
