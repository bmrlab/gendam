use super::AudioTaskType;
use crate::{
    record::{TaskRunOutput, TaskRunRecord},
    ContentTask, ContentTaskType, FileInfo,
};
use ai::AudioTranscriptOutput;
use async_trait::async_trait;
use content_base_context::ContentBaseCtx;
use content_handler::audio::AudioDecoder;
use serde_json::{json, Value};
use std::path::PathBuf;
use storage_macro::Storage;

#[async_trait]
pub trait AudioTranscriptTrait: Into<ContentTaskType> + Clone + Storage {
    async fn audio_path(
        &self,
        file_info: &FileInfo,
        ctx: &ContentBaseCtx,
    ) -> anyhow::Result<PathBuf>;

    async fn transcript_output(
        &self,
        task_run_record: &TaskRunRecord,
    ) -> anyhow::Result<TaskRunOutput> {
        let task_type: ContentTaskType = self.clone().into();
        Ok(TaskRunOutput::File(PathBuf::from(format!(
            "{}-{}.json",
            task_type.to_string(),
            task_run_record.id()
        ))))
    }

    async fn run_audio_transcript(
        &self,
        file_info: &FileInfo,
        ctx: &ContentBaseCtx,
        task_run_record: &mut TaskRunRecord,
    ) -> anyhow::Result<()> {
        let output_path = task_run_record
            .output_path(&file_info.file_identifier, ctx)
            .await?;

        let result = ctx
            .audio_transcript()?
            .0
            .process_single(self.audio_path(file_info, ctx).await?)
            .await?;

        self.write(output_path.clone(), serde_json::to_string(&result)?.into())
            .await?;

        Ok(())
    }

    fn audio_task_parameters(&self, ctx: &ContentBaseCtx) -> Value {
        json!({
            "model": ctx.audio_transcript().expect("audio model is set").1,
        })
    }

    async fn transcript_content(
        &self,
        file_info: &FileInfo,
        ctx: &ContentBaseCtx,
    ) -> anyhow::Result<AudioTranscriptOutput> {
        let task_type: ContentTaskType = self.clone().into();
        let output_path = task_type.task_output_path(file_info, ctx).await?;
        let content = self.read_to_string(output_path)?;
        Ok(serde_json::from_str(&content)?)
    }
}

#[derive(Clone, Storage, Debug, Default)]
pub struct AudioTranscriptTask;

#[async_trait]
impl AudioTranscriptTrait for AudioTranscriptTask {
    async fn audio_path(
        &self,
        file_info: &FileInfo,
        ctx: &ContentBaseCtx,
    ) -> anyhow::Result<PathBuf> {
        let artifacts_dir = ctx.artifacts_dir(&file_info.file_identifier);
        let tmp_audio_path = artifacts_dir.join("tmp.wav");
        Ok(tmp_audio_path)
    }
}

#[async_trait]
impl ContentTask for AudioTranscriptTask {
    async fn task_output(&self, task_run_record: &TaskRunRecord) -> anyhow::Result<TaskRunOutput> {
        self.transcript_output(task_run_record).await
    }

    async fn inner_run(
        &self,
        file_info: &FileInfo,
        ctx: &ContentBaseCtx,
        task_run_record: &mut TaskRunRecord,
    ) -> anyhow::Result<()> {
        let tmp_audio_path = self.audio_path(file_info, ctx).await?;
        let audio_decoder = AudioDecoder::new(&file_info.file_path)?;
        audio_decoder.save_whisper_format(&tmp_audio_path)?;
        self.run_audio_transcript(file_info, ctx, task_run_record)
            .await?;

        if let Err(e) = self.remove_file(tmp_audio_path) {
            tracing::warn!("failed to remove tmp audio file: {e}");
        }

        Ok(())
    }

    fn task_parameters(&self, ctx: &ContentBaseCtx) -> Value {
        self.audio_task_parameters(ctx)
    }

    fn task_dependencies(&self) -> Vec<ContentTaskType> {
        vec![] as Vec<ContentTaskType>
    }
}

impl Into<ContentTaskType> for AudioTranscriptTask {
    fn into(self) -> ContentTaskType {
        ContentTaskType::Audio(AudioTaskType::Transcript(self.clone()))
    }
}
