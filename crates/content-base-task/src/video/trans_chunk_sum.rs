use super::{trans_chunk::VideoTransChunkTask, transcript::VideoTranscriptTask, VideoTaskType};
use crate::{
    audio::{
        trans_chunk::AudioTranscriptChunkTrait, trans_chunk_sum::AudioTransChunkSumTrait,
        transcript::AudioTranscriptTrait,
    },
    record::{TaskRunOutput, TaskRunRecord},
    ContentTask, ContentTaskType,
};
use async_trait::async_trait;
use content_base_context::ContentBaseCtx;
use storage_macro::Storage;

#[derive(Clone, Storage, Debug, Default)]
pub struct VideoTransChunkSumTask;

#[async_trait]
impl AudioTransChunkSumTrait for VideoTransChunkSumTask {
    fn chunk_task(&self) -> impl AudioTranscriptChunkTrait {
        VideoTransChunkTask
    }

    fn transcript_task(&self) -> impl AudioTranscriptTrait {
        VideoTranscriptTask
    }
}

#[async_trait]
impl ContentTask for VideoTransChunkSumTask {
    async fn task_output(&self, task_run_record: &TaskRunRecord) -> anyhow::Result<TaskRunOutput> {
        self.audio_trans_sum_output(task_run_record).await
    }

    async fn inner_run(
        &self,
        file_info: &crate::FileInfo,
        ctx: &ContentBaseCtx,
        task_run_record: &mut crate::record::TaskRunRecord,
    ) -> anyhow::Result<()> {
        self.run_sum(file_info, ctx, task_run_record)
            .await
    }

    fn task_parameters(&self, ctx: &ContentBaseCtx) -> serde_json::Value {
        self.sum_parameters(ctx)
    }

    fn task_dependencies(&self) -> Vec<ContentTaskType> {
        vec![VideoTransChunkTask.into()]
    }
}

impl Into<ContentTaskType> for VideoTransChunkSumTask {
    fn into(self) -> ContentTaskType {
        ContentTaskType::Video(VideoTaskType::TransChunkSum(self.clone()))
    }
}
