use super::ImageTaskType;
use crate::{ContentTask, ContentTaskType, TaskRunOutput, TaskRunRecord};
use async_trait::async_trait;
use content_base_context::ContentBaseCtx;
use serde_json::{json, Value};
use std::path::PathBuf;
use storage_macro::Storage;

#[derive(Clone, Debug, Default, Storage)]
pub struct ImageEmbeddingTask;

#[async_trait]
impl ContentTask for ImageEmbeddingTask {
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

        let (model, _) = ctx.multi_modal_embedding()?;
        let model_input =
            ai::MultiModalEmbeddingInput::Image(file_info.file_full_path_on_disk.clone());
        let model_output = model.process_single(model_input).await?;

        self.write(
            output_path.clone(),
            serde_json::to_string(&model_output)?.into(), // 直接把 Vec<f32> 转成字符串
        )
        .await?;

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
        vec![] as Vec<ContentTaskType>
    }
}

impl Into<ContentTaskType> for ImageEmbeddingTask {
    fn into(self) -> ContentTaskType {
        ContentTaskType::Image(ImageTaskType::Embedding(self.clone()))
    }
}

impl ImageEmbeddingTask {
    pub async fn embedding_content(
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
