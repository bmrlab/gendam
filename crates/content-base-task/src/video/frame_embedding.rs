use super::{frame::VideoFrameTask, VideoTaskType};
use crate::{ContentTask, ContentTaskType, FileInfo, TaskRunOutput, TaskRunRecord};
use async_trait::async_trait;
use content_base_context::ContentBaseCtx;
use serde_json::{json, Value};
use std::path::PathBuf;
use storage_macro::Storage;

#[derive(Clone, Debug, Default, Storage)]
pub struct VideoFrameEmbeddingTask;

#[async_trait]
impl ContentTask for VideoFrameEmbeddingTask {
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
            let (model, _) = ctx.multi_modal_embedding()?;
            let model_input = ai::MultiModalEmbeddingInput::Image(image_absolute_path.clone());
            let model_output = model.process_single(model_input).await?;

            let output_path = output_path.join(format!("{}.json", frame_info.timestamp));
            self.write(
                output_path,
                serde_json::to_string(&model_output)?.into(), // 直接把 Vec<f32> 转成字符串
            )
            .await?;
        }

        Ok(())
    }

    fn task_parameters(&self, ctx: &ContentBaseCtx) -> Value {
        let (_, model_name) = ctx
            .multi_modal_embedding()
            .expect("multi modal embedding model should be set");
        json!({
            "model": model_name,
        })
    }

    fn task_dependencies(&self) -> Vec<ContentTaskType> {
        vec![VideoFrameTask.into()]
    }
}

impl Into<ContentTaskType> for VideoFrameEmbeddingTask {
    fn into(self) -> ContentTaskType {
        ContentTaskType::Video(VideoTaskType::FrameEmbedding(self.clone()))
    }
}

impl VideoFrameEmbeddingTask {
    pub async fn frame_embedding_content(
        &self,
        file_info: &FileInfo,
        ctx: &ContentBaseCtx,
        timestamp: i64,
    ) -> anyhow::Result<Vec<f32>> {
        let task_type: ContentTaskType = self.clone().into();
        let output_path = task_type
            .task_output_path(file_info, ctx)
            .await?
            .join(format!("{}.json", timestamp));
        let content_str = self.read_to_string(output_path)?;
        Ok(serde_json::from_str(&content_str)?)
    }
}
