use std::path::PathBuf;

use super::ImageTaskType;
use crate::{
    image::description::ImageDescriptionTask, ContentTask, ContentTaskType, FileInfo,
    TaskRunOutput, TaskRunRecord,
};
use async_trait::async_trait;
use content_base_context::ContentBaseCtx;
use serde_json::{json, Value};
use storage_macro::Storage;

#[derive(Clone, Debug, Default, Storage)]
pub struct ImageDescEmbedTask;

#[async_trait]
impl ContentTask for ImageDescEmbedTask {
    async fn task_output(&self, task_run_record: &TaskRunRecord) -> anyhow::Result<TaskRunOutput> {
        let task_type: ContentTaskType = self.clone().into();

        Ok(TaskRunOutput::File(PathBuf::from(format!(
            "{}-{}.json",
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
        let description = ImageDescriptionTask
            .description_content(file_info, ctx)
            .await?;

        let output_path = task_run_record
            .output_path(&file_info.file_identifier, ctx)
            .await?;

        ctx.save_text_embedding(&description, &output_path).await?;

        Ok(())
    }

    fn task_parameters(&self, ctx: &ContentBaseCtx) -> Value {
        json!({
            "model": ctx.text_embedding().expect("text embedding is set").1
        })
    }

    fn task_dependencies(&self) -> Vec<ContentTaskType> {
        vec![ImageDescriptionTask.into()]
    }
}

impl Into<ContentTaskType> for ImageDescEmbedTask {
    fn into(self) -> ContentTaskType {
        ContentTaskType::Image(ImageTaskType::DescEmbed(self.clone()))
    }
}

impl ImageDescEmbedTask {
    pub async fn desc_embed_content(
        &self,
        file_info: &FileInfo,
        ctx: &ContentBaseCtx,
    ) -> anyhow::Result<Vec<f32>> {
        let task_type: ContentTaskType = self.clone().into();
        let output_path = task_type.task_output_path(file_info, ctx).await?;

        let content_str = self.read_to_string(output_path)?;

        Ok(serde_json::from_str(&content_str)?)
    }
}
