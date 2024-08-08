use crate::{
    raw_text::{chunk::DocumentChunkTrait, chunk_sum::DocumentChunkSumTrait},
    ContentTask, ContentTaskType, TaskRunOutput, TaskRunRecord,
};
use async_trait::async_trait;
use content_base_context::ContentBaseCtx;
use serde_json::Value;
use storage_macro::Storage;

use super::{chunk::WebPageChunkTask, WebPageTaskType};

#[derive(Clone, Debug, Default, Storage)]
pub struct WebPageChunkSumTask;

impl DocumentChunkSumTrait for WebPageChunkSumTask {
    fn chunk_task(&self) -> impl DocumentChunkTrait {
        WebPageChunkTask
    }
}

#[async_trait]
impl ContentTask for WebPageChunkSumTask {
    async fn task_output(&self, task_run_record: &TaskRunRecord) -> anyhow::Result<TaskRunOutput> {
        self.chunk_sum_output(task_run_record).await
    }

    async fn inner_run(
        &self,
        file_info: &crate::FileInfo,
        ctx: &ContentBaseCtx,
        task_run_record: &mut TaskRunRecord,
    ) -> anyhow::Result<()> {
        self.run_sum(file_info, ctx, task_run_record).await
    }

    fn task_parameters(&self, ctx: &ContentBaseCtx) -> Value {
        self.sum_parameters(ctx)
    }

    fn task_dependencies(&self) -> Vec<ContentTaskType> {
        vec![WebPageChunkTask.into()]
    }
}

impl Into<ContentTaskType> for WebPageChunkSumTask {
    fn into(self) -> ContentTaskType {
        ContentTaskType::WebPage(WebPageTaskType::ChunkSum(self.clone()))
    }
}
