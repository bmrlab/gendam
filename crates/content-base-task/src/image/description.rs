use super::ImageTaskType;
use crate::{ContentTask, ContentTaskType, TaskRunOutput, TaskRunRecord};
use async_trait::async_trait;
use content_base_context::ContentBaseCtx;
use serde_json::{json, Value};
use std::path::PathBuf;
use storage_macro::Storage;

#[derive(Clone, Debug, Default, Storage)]
pub struct ImageDescriptionTask;

#[async_trait]
impl ContentTask for ImageDescriptionTask {
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
        file_info: &crate::FileInfo,
        ctx: &ContentBaseCtx,
        task_run_record: &mut TaskRunRecord,
    ) -> anyhow::Result<()> {
        let output_path = task_run_record
            .output_path(&file_info.file_identifier, ctx)
            .await?;

        let result = ctx
            .image_caption()?
            .0
            .process_single(file_info.file_path.clone())
            .await?;

        self.write(
            output_path.clone(),
            serde_json::to_string(&json!({
                "caption": result
            }))?
            .into(),
        )
        .await?;

        Ok(())
    }

    fn task_parameters(&self, ctx: &ContentBaseCtx) -> Value {
        json!({
            "model": ctx.image_caption().expect("image caption model is set").1,
        })
    }

    fn task_dependencies(&self) -> Vec<ContentTaskType> {
        vec![] as Vec<ContentTaskType>
    }
}

impl Into<ContentTaskType> for ImageDescriptionTask {
    fn into(self) -> ContentTaskType {
        ContentTaskType::Image(ImageTaskType::Description(self.clone()))
    }
}

impl ImageDescriptionTask {
    pub async fn description_content(
        &self,
        file_info: &crate::FileInfo,
        ctx: &ContentBaseCtx,
    ) -> anyhow::Result<String> {
        let task_type: ContentTaskType = self.clone().into();
        let output_path = task_type.task_output_path(file_info, ctx).await?;
        let content = self.read_to_string(output_path)?;
        let content: Value = serde_json::from_str(&content)?;

        let content = content.get("caption").ok_or(anyhow::anyhow!(
            "no description found in image caption file"
        ))?;
        let content = content.as_str().ok_or(anyhow::anyhow!(
            "no description found in image caption file"
        ))?;

        Ok(content.to_string())
    }
}
