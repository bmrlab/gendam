use super::{chunk::WebPageChunkTask, chunk_sum::WebPageChunkSumTask, WebPageTaskType};
use crate::{
    raw_text::{
        chunk::DocumentChunkTrait, chunk_sum::DocumentChunkSumTrait,
        chunk_sum_embed::DocumentChunkSumEmbedTrait,
    },
    ContentTask, ContentTaskType, TaskRunOutput, TaskRunRecord,
};
use async_trait::async_trait;
use content_base_context::ContentBaseCtx;
use serde_json::Value;
use storage_macro::Storage;

#[derive(Clone, Debug, Default, Storage)]
pub struct WebPageChunkSumEmbedTask;

#[async_trait]
impl DocumentChunkSumEmbedTrait for WebPageChunkSumEmbedTask {
    fn sum_task(&self) -> impl DocumentChunkSumTrait {
        WebPageChunkSumTask
    }
    fn chunk_task(&self) -> impl DocumentChunkTrait {
        WebPageChunkTask
    }
}

#[async_trait]
impl ContentTask for WebPageChunkSumEmbedTask {
    async fn task_output(&self, task_run_record: &TaskRunRecord) -> anyhow::Result<TaskRunOutput> {
        self.embed_output(task_run_record).await
    }

    async fn inner_run(
        &self,
        file_info: &crate::FileInfo,
        ctx: &ContentBaseCtx,
        task_run_record: &mut TaskRunRecord,
    ) -> anyhow::Result<()> {
        self.run_sum_embed(file_info, ctx, task_run_record).await
    }

    fn task_parameters(&self, ctx: &ContentBaseCtx) -> Value {
        self.embed_parameters(ctx)
    }

    fn task_dependencies(&self) -> Vec<ContentTaskType> {
        vec![WebPageChunkSumTask.into()]
    }
}

impl Into<ContentTaskType> for WebPageChunkSumEmbedTask {
    fn into(self) -> ContentTaskType {
        ContentTaskType::WebPage(WebPageTaskType::ChunkSumEmbed(self.clone()))
    }
}
