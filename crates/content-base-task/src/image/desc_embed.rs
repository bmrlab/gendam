use std::path::PathBuf;

use super::{description::ImageDescriptionTask, ImageTaskType};
use crate::{ContentTask, ContentTaskType, FileInfo, TaskRunOutput, TaskRunRecord};
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
            .description_content(&file_info.file_identifier, ctx)
            .await?;

        let output_path = task_run_record
            .output_path(&file_info.file_identifier, ctx)
            .await?;

        ctx.save_text_embedding(&description, &output_path).await?;

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
        file_identifier: &str,
        ctx: &ContentBaseCtx,
    ) -> anyhow::Result<Vec<f32>> {
        let task_type: ContentTaskType = self.clone().into();
        let output_path = task_type.task_output_path(file_identifier, ctx).await?;

        let content_str = self.read_to_string(output_path)?;

        Ok(serde_json::from_str(&content_str)?)
    }
}
