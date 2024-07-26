use super::{
    trans_chunk::VideoTransChunkTask, trans_chunk_sum::VideoTransChunkSumTask, VideoTaskType,
};
use crate::{
    audio::{
        trans_chunk::AudioTranscriptChunkTrait, trans_chunk_sum::AudioTransChunkSumTrait,
        trans_chunk_sum_embed::AudioTransChunkSumEmbedTrait,
    },
    record::{TaskRunOutput, TaskRunRecord},
    ContentTask, ContentTaskType,
};
use async_trait::async_trait;
use content_base_context::ContentBaseCtx;
use storage_macro::Storage;

#[derive(Clone, Storage, Debug, Default)]
pub struct VideoTransChunkSumEmbedTask;

#[async_trait]
impl AudioTransChunkSumEmbedTrait for VideoTransChunkSumEmbedTask {
    fn chunk_task(&self) -> impl AudioTranscriptChunkTrait {
        VideoTransChunkTask
    }

    fn sum_task(&self) -> impl AudioTransChunkSumTrait {
        VideoTransChunkSumTask
    }
}

#[async_trait]
impl ContentTask for VideoTransChunkSumEmbedTask {
    async fn task_output(&self, task_run_record: &TaskRunRecord) -> anyhow::Result<TaskRunOutput> {
        self.embed_output(task_run_record).await
    }

    async fn inner_run(
        &self,
        file_info: &crate::FileInfo,
        ctx: &ContentBaseCtx,
        task_run_record: &mut crate::record::TaskRunRecord,
    ) -> anyhow::Result<()> {
        self.run_sum_embed(file_info, ctx, task_run_record).await
    }

    fn task_parameters(&self, ctx: &ContentBaseCtx) -> serde_json::Value {
        self.embed_parameters(ctx)
    }

    fn task_dependencies(&self) -> Vec<ContentTaskType> {
        vec![VideoTransChunkSumTask.into()]
    }
}

impl Into<ContentTaskType> for VideoTransChunkSumEmbedTask {
    fn into(self) -> ContentTaskType {
        ContentTaskType::Video(VideoTaskType::TransChunkSumEmbed(self.clone()))
    }
}
