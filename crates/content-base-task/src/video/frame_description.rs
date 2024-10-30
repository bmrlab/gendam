use super::{frame::VideoFrameTask, VideoTaskType};
use crate::{ContentTask, ContentTaskType, TaskRunOutput, TaskRunRecord};
use async_trait::async_trait;
use content_base_context::ContentBaseCtx;
use serde_json::{json, Value};
use std::path::PathBuf;
use storage_macro::Storage;

#[derive(Clone, Debug, Default, Storage)]
pub struct VideoFrameDescriptionTask;

#[async_trait]
impl ContentTask for VideoFrameDescriptionTask {
    async fn task_output(&self, task_run_record: &TaskRunRecord) -> anyhow::Result<TaskRunOutput> {
        let task_type: ContentTaskType = self.clone().into();
        Ok(TaskRunOutput::Folder(PathBuf::from(format!(
            "{}-{}.json",
            task_type.to_string(),
            task_run_record.id()
        ))))
    }

    async fn inner_run(
        &self,
        file_info: &crate::FileInfo,
        ctx: &ContentBaseCtx,
        task_run_record: &mut TaskRunRecord,
    ) -> anyhow::Result<()> {
        let output_path = task_run_record
            .output_path(&file_info.file_identifier, ctx)
            .await?;

        let frame_infos = VideoFrameTask.frame_content(file_info, ctx).await?;
        for frame_info in frame_infos {
            let image_absolute_path = self
                .get_absolute_path(frame_info.image_file.clone())
                .map_err(|e| {
                    anyhow::anyhow!(
                        "Failed to get absolute path for frame image file {:?}: {:?}",
                        frame_info.image_file.clone(),
                        e
                    )
                })?;
            let (model, _) = ctx.image_caption()?;
            let model_input: ai::ImageCaptionInput = image_absolute_path.clone();
            let model_output = model.process_single(model_input).await?;

            let output_path = output_path.join(format!("{}.json", frame_info.timestamp));
            let json_output = serde_json::to_string(&json!({
                "caption": model_output
            }))?;
            self.write(output_path, json_output.into()).await?;
        }

        Ok(())
    }

    fn task_parameters(&self, ctx: &ContentBaseCtx) -> Value {
        let (_, model_name) = ctx
            .multi_modal_embedding()
            .expect("image caption model should be set");
        json!({
            "model": model_name,
        })
    }

    fn task_dependencies(&self) -> Vec<ContentTaskType> {
        // VideoFrameEmbeddingTask 也设置了同样的依赖，需要确认这会不会导致 VideoFrameTask 重复执行
        vec![VideoFrameTask.into()]
    }
}

impl Into<ContentTaskType> for VideoFrameDescriptionTask {
    fn into(self) -> ContentTaskType {
        ContentTaskType::Video(VideoTaskType::FrameDescription(self.clone()))
    }
}

impl VideoFrameDescriptionTask {
    pub async fn frame_description_content(
        &self,
        file_info: &crate::FileInfo,
        ctx: &ContentBaseCtx,
        timestamp: i64,
    ) -> anyhow::Result<String> {
        let task_type: ContentTaskType = self.clone().into();
        let output_path = task_type
            .task_output_path(file_info, ctx)
            .await?
            .join(format!("{}.json", timestamp));
        let file_content = self.read_to_string(output_path)?;
        let json_content: Value = serde_json::from_str(&file_content)?;

        let caption = json_content
            .get("caption")
            .ok_or(anyhow::anyhow!(
                "caption field not found in image caption file"
            ))?
            .as_str()
            .ok_or(anyhow::anyhow!(
                "caption field is not a string in image caption file"
            ))?;

        Ok(caption.to_string())
    }
}
