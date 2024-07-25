use std::path::PathBuf;
use super::{audio::VideoAudioTask, VideoTaskType};
use crate::{
    audio::transcript::AudioTranscriptTrait,
    record::{TaskRunOutput, TaskRunRecord},
    ContentTask, ContentTaskType, FileInfo,
};
use async_trait::async_trait;
use content_base_context::ContentBaseCtx;
use storage_macro::Storage;

#[derive(Clone, Storage, Debug, Default)]
pub struct VideoTranscriptTask;

#[async_trait]
impl AudioTranscriptTrait for VideoTranscriptTask {
    async fn audio_path(&self, file_info: &FileInfo, ctx: &ContentBaseCtx) -> anyhow::Result<PathBuf> {
        VideoAudioTask.task_output_path(file_info, ctx).await
    }
}

#[async_trait]
impl ContentTask for VideoTranscriptTask {
    async fn task_output(&self, task_run_record: &TaskRunRecord) -> anyhow::Result<TaskRunOutput> {
        self.transcript_output(task_run_record).await
    }

    async fn inner_run(
        &self,
        file_info: &FileInfo,
        ctx: &ContentBaseCtx,
        task_run_record: &mut TaskRunRecord,
    ) -> anyhow::Result<()> {
        self.run_audio_transcript(file_info, ctx, task_run_record)
            .await
    }

    fn task_parameters(&self, ctx: &ContentBaseCtx) -> serde_json::Value {
        self.audio_task_parameters(ctx)
    }

    fn task_dependencies(&self) -> Vec<ContentTaskType> {
        vec![VideoAudioTask.into()]
    }
}

impl Into<ContentTaskType> for VideoTranscriptTask {
    fn into(self) -> ContentTaskType {
        ContentTaskType::Video(VideoTaskType::Transcript(self.clone()))
    }
}
