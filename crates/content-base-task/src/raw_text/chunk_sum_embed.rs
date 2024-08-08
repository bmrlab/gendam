use std::path::PathBuf;
use async_trait::async_trait;
use content_base_context::ContentBaseCtx;
use serde_json::{json, Value};
use storage_macro::Storage;
use crate::{ContentTask, ContentTaskType, TaskRunOutput, TaskRunRecord};
use super::{
    chunk::{DocumentChunkTrait, RawTextChunkTask},
    chunk_sum::{DocumentChunkSumTrait, RawTextChunkSumTask},
    RawTextTaskType,
};

#[derive(Clone, Debug, Default, Storage)]
pub struct RawTextChunkSumEmbedTask;

#[async_trait]
pub trait DocumentChunkSumEmbedTrait: Into<ContentTaskType> + Clone + Storage {
    fn chunk_task(&self) -> impl DocumentChunkTrait;
    fn sum_task(&self) -> impl DocumentChunkSumTrait;

    async fn run_sum_embed(
        &self,
        file_info: &crate::FileInfo,
        ctx: &ContentBaseCtx,
        task_run_record: &mut crate::record::TaskRunRecord,
    ) -> anyhow::Result<()> {
        let chunks = self.chunk_task().chunk_content(file_info, ctx).await?;
        for idx in 0..chunks.len() {
            let summarization = self.sum_task().sum_content(file_info, ctx, idx).await?;

            let output_dir = task_run_record
                .output_path(&file_info.file_identifier, ctx)
                .await?;
            let output_path = output_dir.join(format!("{}.{}", idx, "json"));

            ctx.save_text_embedding(&summarization, &output_path)
                .await?;
        }

        Ok(())
    }

    async fn embed_output(&self, task_run_record: &TaskRunRecord) -> anyhow::Result<TaskRunOutput> {
        let task_type: ContentTaskType = self.clone().into();
        Ok(TaskRunOutput::Folder(PathBuf::from(format!(
            "{}-{}",
            task_type.to_string(),
            task_run_record.id()
        ))))
    }

    fn embed_parameters(&self, ctx: &ContentBaseCtx) -> Value {
        json!({
            "model": ctx.text_embedding().expect("text embedding is set").1
        })
    }

    async fn embed_content(
        &self,
        file_info: &crate::FileInfo,
        ctx: &ContentBaseCtx,
        index: usize,
    ) -> anyhow::Result<Vec<f32>> {
        let task_type: ContentTaskType = self.clone().into();
        let output_path = task_type
            .task_output_path(file_info, ctx)
            .await?
            .join(format!("{}.json", index));
        let content_str = self.read_to_string(output_path)?;
        let embedding: Vec<f32> = serde_json::from_str(&content_str)?;

        Ok(embedding)
    }
}

#[async_trait]
impl DocumentChunkSumEmbedTrait for RawTextChunkSumEmbedTask {
    fn sum_task(&self) -> impl DocumentChunkSumTrait {
        RawTextChunkSumTask
    }
    fn chunk_task(&self) -> impl DocumentChunkTrait {
        RawTextChunkTask
    }
}

#[async_trait]
impl ContentTask for RawTextChunkSumEmbedTask {
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
        vec![RawTextChunkSumTask.into()]
    }
}

impl Into<ContentTaskType> for RawTextChunkSumEmbedTask {
    fn into(self) -> ContentTaskType {
        ContentTaskType::RawText(RawTextTaskType::ChunkSumEmbed(self.clone()))
    }
}
