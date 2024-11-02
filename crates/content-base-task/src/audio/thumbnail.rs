use super::AudioTaskType;
use crate::{ContentTask, ContentTaskType, FileInfo, TaskRunOutput, TaskRunRecord};
use async_trait::async_trait;
use content_base_context::ContentBaseCtx;
use content_handler::audio::AudioDecoder;
use serde_json::{json, Value};
use std::path::PathBuf;
use storage_macro::Storage;

#[derive(Clone, Debug, Default, Storage)]
pub struct AudioThumbnailTask;

#[async_trait]
pub trait AudioThumbnailTrait: Into<ContentTaskType> + Clone + Storage {
    async fn audio_thumbnail_output(
        &self,
        _task_run_record: &TaskRunRecord,
    ) -> anyhow::Result<TaskRunOutput> {
        Ok(TaskRunOutput::File(PathBuf::from(
            format!("thumbnail.jpg",),
        )))
    }

    async fn run_audio_thumbnail(
        &self,
        file_info: &FileInfo,
        ctx: &ContentBaseCtx,
        task_run_record: &mut TaskRunRecord,
    ) -> anyhow::Result<()> {
        let audio_decoder = AudioDecoder::new(&file_info.file_full_path_on_disk)?;
        let output_path = task_run_record
            .output_path(&file_info.file_identifier, ctx)
            .await?;

        // 封面图提取失败不算失败，因为音频可能没有封面图
        if let Err(e) = audio_decoder.save_audio_cover(output_path) {
            tracing::warn!("failed to save audio thumbnail: {e:?}");
        }

        Ok(())
    }

    fn audio_thumbnail_parameters(&self, _ctx: &ContentBaseCtx) -> Value {
        json!({
            "method":"ffmpeg"
        })
    }
}

#[async_trait]
impl AudioThumbnailTrait for AudioThumbnailTask {}

#[async_trait]
impl ContentTask for AudioThumbnailTask {
    async fn task_output(&self, task_run_record: &TaskRunRecord) -> anyhow::Result<TaskRunOutput> {
        self.audio_thumbnail_output(task_run_record).await
    }

    async fn inner_run(
        &self,
        file_info: &FileInfo,
        ctx: &ContentBaseCtx,
        task_run_record: &mut TaskRunRecord,
    ) -> anyhow::Result<()> {
        self.run_audio_thumbnail(file_info, ctx, task_run_record)
            .await
    }

    fn task_parameters(&self, ctx: &ContentBaseCtx) -> Value {
        self.audio_thumbnail_parameters(ctx)
    }

    fn task_dependencies(&self) -> Vec<ContentTaskType> {
        vec![] as Vec<ContentTaskType>
    }
}

impl Into<ContentTaskType> for AudioThumbnailTask {
    fn into(self) -> ContentTaskType {
        ContentTaskType::Audio(AudioTaskType::Thumbnail(self.clone()))
    }
}
