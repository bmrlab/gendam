use super::{
    frame::{VideoFrameTask, VIDEO_FRAME_SUMMARY_BATCH_SIZE},
    frame_description::VideoFrameDescriptionTask,
    VideoTaskType,
};
use crate::{ContentTask, ContentTaskType, FileInfo, TaskRunOutput, TaskRunRecord};
use async_trait::async_trait;
use content_base_context::ContentBaseCtx;
use serde_json::{json, Value};
use std::path::PathBuf;
use storage_macro::Storage;

#[derive(Clone, Debug, Default, Storage)]
pub struct VideoFrameDescEmbedTask;

#[async_trait]
impl ContentTask for VideoFrameDescEmbedTask {
    async fn task_output(&self, task_run_record: &TaskRunRecord) -> anyhow::Result<TaskRunOutput> {
        let task_type: ContentTaskType = self.clone().into();
        Ok(TaskRunOutput::Folder(PathBuf::from(format!(
            "{}-{}",
            task_type.to_string(),
            task_run_record.id()
        ))))
    }

    async fn inner_run(
        &self,
        file_info: &FileInfo,
        ctx: &ContentBaseCtx,
        task_run_record: &mut TaskRunRecord,
    ) -> anyhow::Result<()> {
        let output_path = task_run_record
            .output_path(&file_info.file_identifier, ctx)
            .await?;

        let frame_infos = VideoFrameTask.frame_content(file_info, ctx).await?;
        for frame_infos_chunk in frame_infos.chunks(VIDEO_FRAME_SUMMARY_BATCH_SIZE) {
            let first_frame = frame_infos_chunk.first().expect("first frame should exist");
            let last_frame = frame_infos_chunk.last().expect("last frame should exist");
            let description = VideoFrameDescriptionTask
                .frame_description_content(
                    file_info,
                    ctx,
                    first_frame.timestamp,
                    last_frame.timestamp,
                )
                .await?;
            let output_path = output_path.join(format!(
                "{}-{}.json",
                first_frame.timestamp, last_frame.timestamp
            ));
            ctx.save_text_embedding(&description, &output_path).await?;
        }

        Ok(())
    }

    fn task_parameters(&self, ctx: &ContentBaseCtx) -> Value {
        let (_, model_name) = ctx
            .multi_modal_embedding()
            .expect("text embedding model should be set");
        json!({
            "model": model_name,
        })
    }

    fn task_dependencies(&self) -> Vec<ContentTaskType> {
        vec![VideoFrameDescriptionTask.into()]
    }
}

impl Into<ContentTaskType> for VideoFrameDescEmbedTask {
    fn into(self) -> ContentTaskType {
        ContentTaskType::Video(VideoTaskType::FrameDescEmbed(self.clone()))
    }
}

impl VideoFrameDescEmbedTask {
    pub async fn frame_desc_embed_content(
        &self,
        file_info: &FileInfo,
        ctx: &ContentBaseCtx,
        start_timestamp: i64,
        end_timestamp: i64,
    ) -> anyhow::Result<Vec<f32>> {
        let task_type: ContentTaskType = self.clone().into();
        let output_path = task_type
            .task_output_path(file_info, ctx)
            .await?
            .join(format!("{}-{}.json", start_timestamp, end_timestamp));
        let content_str = self.read_to_string(output_path)?;
        Ok(serde_json::from_str(&content_str)?)
    }
}
