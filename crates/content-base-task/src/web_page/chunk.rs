use crate::{
    raw_text::chunk::DocumentChunkTrait, ContentTask, ContentTaskType, FileInfo, TaskRunOutput,
    TaskRunRecord,
};
use async_trait::async_trait;
use content_base_context::ContentBaseCtx;
use serde_json::Value;
use storage_macro::Storage;

use super::{transform::WebPageTransformTask, WebPageTaskType};

#[derive(Clone, Debug, Default, Storage)]
pub struct WebPageChunkTask;

#[async_trait]
impl DocumentChunkTrait for WebPageChunkTask {
    async fn text_content(
        &self,
        file_info: &FileInfo,
        ctx: &ContentBaseCtx,
    ) -> anyhow::Result<String> {
        WebPageTransformTask.markdown_content(file_info, ctx).await
    }
}

#[async_trait]
impl ContentTask for WebPageChunkTask {
    async fn task_output(&self, task_run_record: &TaskRunRecord) -> anyhow::Result<TaskRunOutput> {
        self.chunk_output(task_run_record).await
    }

    async fn inner_run(
        &self,
        file_info: &FileInfo,
        ctx: &ContentBaseCtx,
        task_run_record: &mut TaskRunRecord,
    ) -> anyhow::Result<()> {
        self.run_chunk(file_info, ctx, task_run_record).await
    }

    fn task_parameters(&self, _ctx: &ContentBaseCtx) -> Value {
        self.chunk_parameters(_ctx)
    }

    fn task_dependencies(&self) -> Vec<ContentTaskType> {
        vec![WebPageTransformTask.into()] as Vec<ContentTaskType>
    }
}

impl Into<ContentTaskType> for WebPageChunkTask {
    fn into(self) -> ContentTaskType {
        ContentTaskType::WebPage(WebPageTaskType::Chunk(self.clone()))
    }
}
