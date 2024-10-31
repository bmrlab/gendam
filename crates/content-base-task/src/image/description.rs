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

        let (model, _) = ctx.image_caption()?;
        let model_input = ai::ImageCaptionInput {
            image_file_paths: vec![file_info.file_path.clone()],
            prompt: Some(
                r#"You are an advanced image analysis AI. Examine the image and describe its contents in a concise, text-only format. Focus on identifying: People (including celebrities), actions, objects, animals or pets, nature elements, visual cues of sounds, human speech (if text bubbles present), displayed text (OCR), and brand logos. Provide specific examples for each category found in the image. Only mention categories that are present; omit any that are not detected. Use plain text format without lists or JSON. Be accurate and concise in your descriptions. Limit your response to no more than 50 words."#.to_string()
            ),
        };
        let model_output = model.process_single(model_input).await?;
        let json_output = serde_json::to_string(&json!({
            "caption": model_output
        }))?;
        self.write(output_path, json_output.into()).await?;

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
