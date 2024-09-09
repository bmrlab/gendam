use super::AudioTaskType;
use crate::{ContentTask, ContentTaskType, FileInfo, TaskRunOutput, TaskRunRecord};
use async_trait::async_trait;
use content_base_context::ContentBaseCtx;
use content_handler::audio::AudioDecoder;
use serde_json::{json, Value};
use std::path::PathBuf;
use storage_macro::Storage;

#[derive(Clone, Debug, Default, Storage)]
pub struct AudioWaveformTask;

#[async_trait]
pub trait AudioWaveformTrait: Into<ContentTaskType> + Clone + Storage {
    async fn audio_path(
        &self,
        file_info: &FileInfo,
        ctx: &ContentBaseCtx,
    ) -> anyhow::Result<PathBuf>;

    async fn audio_waveform_output(
        &self,
        _task_run_record: &TaskRunRecord,
    ) -> anyhow::Result<TaskRunOutput> {
        Ok(TaskRunOutput::File(PathBuf::from(
            format!("waveform.json",),
        )))
    }

    async fn run_audio_waveform(
        &self,
        file_info: &FileInfo,
        ctx: &ContentBaseCtx,
        task_run_record: &mut TaskRunRecord,
    ) -> anyhow::Result<()> {
        let audio_decoder = AudioDecoder::new(&file_info.file_path)?;
        let waveform = audio_decoder.generate_audio_waveform(500)?;

        let output_path = task_run_record
            .output_path(&file_info.file_identifier, ctx)
            .await?;
        self.write(output_path, serde_json::to_string(&waveform)?.into())
            .await?;

        Ok(())
    }

    fn audio_waveform_parameters(&self, _ctx: &ContentBaseCtx) -> Value {
        json!({
            "samples": "500"
        })
    }

    async fn audio_waveform_content(
        &self,
        file_info: &FileInfo,
        ctx: &ContentBaseCtx,
    ) -> anyhow::Result<Vec<f32>> {
        let task_type: ContentTaskType = self.clone().into();
        let output_path = task_type.task_output_path(file_info, ctx).await?;
        let content = self.read_to_string(output_path)?;
        Ok(serde_json::from_str(&content)?)
    }
}

#[async_trait]
impl AudioWaveformTrait for AudioWaveformTask {
    async fn audio_path(
        &self,
        file_info: &FileInfo,
        _ctx: &ContentBaseCtx,
    ) -> anyhow::Result<PathBuf> {
        Ok(file_info.file_path.clone())
    }
}

#[async_trait]
impl ContentTask for AudioWaveformTask {
    async fn task_output(&self, task_run_record: &TaskRunRecord) -> anyhow::Result<TaskRunOutput> {
        self.audio_waveform_output(task_run_record).await
    }

    async fn inner_run(
        &self,
        file_info: &FileInfo,
        ctx: &ContentBaseCtx,
        task_run_record: &mut TaskRunRecord,
    ) -> anyhow::Result<()> {
        self.run_audio_waveform(file_info, ctx, task_run_record)
            .await
    }

    fn task_parameters(&self, ctx: &ContentBaseCtx) -> Value {
        self.audio_waveform_parameters(ctx)
    }

    fn task_dependencies(&self) -> Vec<ContentTaskType> {
        vec![] as Vec<ContentTaskType>
    }
}

impl Into<ContentTaskType> for AudioWaveformTask {
    fn into(self) -> ContentTaskType {
        ContentTaskType::Audio(AudioTaskType::Waveform(self.clone()))
    }
}
