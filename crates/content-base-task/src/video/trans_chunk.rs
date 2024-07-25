use super::{transcript::VideoTranscriptTask, VideoTaskType};
use crate::{
    audio::trans_chunk::AudioTranscriptChunkTrait,
    record::{TaskRunOutput, TaskRunRecord},
    ContentTask, ContentTaskType,
};
use async_trait::async_trait;
use content_base_context::ContentBaseCtx;
use storage_macro::Storage;

#[derive(Clone, Storage, Debug, Default)]
pub struct VideoTransChunkTask;

#[async_trait]
impl AudioTranscriptChunkTrait for VideoTransChunkTask {
    fn transcript_task(&self) -> impl ContentTask {
        VideoTranscriptTask
    }
}

#[async_trait]
impl ContentTask for VideoTransChunkTask {
    async fn task_output(&self, task_run_record: &TaskRunRecord) -> anyhow::Result<TaskRunOutput> {
        self.chunk_output(task_run_record).await
    }

    async fn inner_run(
        &self,
        file_info: &crate::FileInfo,
        ctx: &ContentBaseCtx,
        task_run_record: &mut crate::record::TaskRunRecord,
    ) -> anyhow::Result<()> {
        self.run_chunk(file_info, ctx, task_run_record)
            .await
    }

    fn task_parameters(&self, ctx: &ContentBaseCtx) -> serde_json::Value {
        self.chunk_parameters(ctx)
    }

    fn task_dependencies(&self) -> Vec<ContentTaskType> {
        vec![VideoTranscriptTask.into()]
    }
}

impl Into<ContentTaskType> for VideoTransChunkTask {
    fn into(self) -> ContentTaskType {
        ContentTaskType::Video(VideoTaskType::TransChunk(self.clone()))
    }
}
