use super::WebPageTaskType;
use crate::{ContentTask, ContentTaskType, FileInfo, TaskRunOutput, TaskRunRecord};
use async_trait::async_trait;
use content_base_context::ContentBaseCtx;
use content_handler::web_page::convert_to_markdown;
use serde_json::{json, Value};
use std::path::PathBuf;
use storage_macro::Storage;

#[derive(Clone, Debug, Default, Storage)]
pub struct WebPageTransformTask;

#[async_trait]
impl ContentTask for WebPageTransformTask {
    async fn task_output(&self, _task_run_record: &TaskRunRecord) -> anyhow::Result<TaskRunOutput> {
        Ok(TaskRunOutput::File(PathBuf::from("transformed.md")))
    }

    async fn inner_run(
        &self,
        file_info: &FileInfo,
        ctx: &ContentBaseCtx,
        task_run_record: &mut TaskRunRecord,
    ) -> anyhow::Result<()> {
        let markdown_string = convert_to_markdown(&file_info.file_path)?;

        let output = self.task_output(task_run_record).await?;
        let output_path = output.to_path_buf(&file_info.file_identifier, ctx).await?;

        let markdown_buffer: Vec<u8> = markdown_string.as_bytes().into();
        self.write(output_path, markdown_buffer.into()).await?;

        Ok(())
    }

    fn task_parameters(&self, _ctx: &ContentBaseCtx) -> Value {
        json!({
            "method": "htmd"
        })
    }

    fn task_dependencies(&self) -> Vec<ContentTaskType> {
        vec![] as Vec<ContentTaskType>
    }
}

impl Into<ContentTaskType> for WebPageTransformTask {
    fn into(self) -> ContentTaskType {
        ContentTaskType::WebPage(WebPageTaskType::Transform(self.clone()))
    }
}

impl WebPageTransformTask {
    pub async fn markdown_content(
        &self,
        file_info: &FileInfo,
        ctx: &ContentBaseCtx,
    ) -> anyhow::Result<String> {
        let path = self.task_output_path(file_info, ctx).await?;

        Ok(self.read_to_string(path)?)
    }
}
