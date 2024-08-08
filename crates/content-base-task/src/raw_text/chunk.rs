use super::RawTextTaskType;
use crate::{ContentTask, ContentTaskType, FileInfo, TaskRunOutput, TaskRunRecord};
use async_trait::async_trait;
use content_base_context::ContentBaseCtx;
use content_handler::doc::chunk::naive_chunk;
use serde_json::{json, Value};
use std::{io::Read, path::PathBuf};
use storage_macro::Storage;

#[derive(Clone, Debug, Default, Storage)]
pub struct RawTextChunkTask;

#[async_trait]
pub trait DocumentChunkTrait: Into<ContentTaskType> + Clone + Storage {
    async fn text_content(
        &self,
        file_info: &FileInfo,
        ctx: &ContentBaseCtx,
    ) -> anyhow::Result<String>;

    async fn chunk_output(&self, task_run_record: &TaskRunRecord) -> anyhow::Result<TaskRunOutput> {
        let task_type: ContentTaskType = self.clone().into();
        Ok(TaskRunOutput::File(PathBuf::from(format!(
            "{}-{}.json",
            task_type.to_string(),
            task_run_record.id()
        ))))
    }

    async fn run_chunk(
        &self,
        file_info: &FileInfo,
        ctx: &ContentBaseCtx,
        task_run_record: &TaskRunRecord,
    ) -> anyhow::Result<()> {
        let tokenizer = ctx.llm_tokenizer()?;

        // TODO need to handle chunk with better strategy
        let content = self.text_content(file_info, ctx).await?;
        let mut items = vec![];
        let mut start = 0;
        for paragraph in content.split("\n").into_iter() {
            items.push(paragraph);
            start = start + paragraph.len() + 1;
        }
        let chunks = naive_chunk(&items, tokenizer, 100)?;
        let chunks = chunks.into_iter().map(|v| v.join("\n")).collect::<Vec<_>>();

        let output_path = task_run_record
            .output_path(&file_info.file_identifier, ctx)
            .await?;
        self.write(output_path, serde_json::to_string(&chunks)?.into())
            .await?;

        Ok(())
    }

    fn chunk_parameters(&self, _ctx: &ContentBaseCtx) -> Value {
        json!({
            "method": "naive"
        })
    }

    async fn chunk_content(
        &self,
        file_info: &FileInfo,
        ctx: &ContentBaseCtx,
    ) -> anyhow::Result<Vec<String>> {
        let task_type: ContentTaskType = self.clone().into();
        let output_path = task_type.task_output_path(file_info, ctx).await?;
        let content = self.read_to_string(output_path)?;
        Ok(serde_json::from_str(&content)?)
    }
}

#[async_trait]
impl DocumentChunkTrait for RawTextChunkTask {
    async fn text_content(
        &self,
        file_info: &FileInfo,
        _ctx: &ContentBaseCtx,
    ) -> anyhow::Result<String> {
        let file = std::fs::File::open(&file_info.file_path)?;
        let mut reader = std::io::BufReader::new(file);
        let mut content = String::new();
        reader.read_to_string(&mut content)?;
        Ok(content)
    }
}

#[async_trait]
impl ContentTask for RawTextChunkTask {
    async fn task_output(&self, task_run_record: &TaskRunRecord) -> anyhow::Result<TaskRunOutput> {
        self.chunk_output(task_run_record).await
    }

    async fn inner_run(
        &self,
        file_info: &FileInfo,
        ctx: &ContentBaseCtx,
        task_run_record: &mut TaskRunRecord,
    ) -> anyhow::Result<()> {
        self.run_chunk(file_info, ctx, task_run_record).await
    }

    fn task_parameters(&self, _ctx: &ContentBaseCtx) -> Value {
        self.chunk_parameters(_ctx)
    }

    fn task_dependencies(&self) -> Vec<ContentTaskType> {
        vec![] as Vec<ContentTaskType>
    }
}

impl Into<ContentTaskType> for RawTextChunkTask {
    fn into(self) -> ContentTaskType {
        ContentTaskType::RawText(RawTextTaskType::Chunk(self.clone()))
    }
}
