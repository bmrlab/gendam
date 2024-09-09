use super::{
    chunk::{DocumentChunkTrait, RawTextChunkTask},
    RawTextTaskType,
};
use crate::{ContentTask, ContentTaskType, TaskRunOutput, TaskRunRecord};
use ai::llm::{LLMInferenceParams, LLMMessage};
use async_trait::async_trait;
use content_base_context::ContentBaseCtx;
use serde_json::{json, Value};
use std::path::PathBuf;
use storage_macro::Storage;

#[async_trait]
pub trait DocumentChunkSumTrait: Into<ContentTaskType> + Clone + Storage {
    fn chunk_task(&self) -> impl DocumentChunkTrait;

    async fn chunk_sum_output(
        &self,
        task_run_record: &TaskRunRecord,
    ) -> anyhow::Result<TaskRunOutput> {
        let task_type: ContentTaskType = self.clone().into();
        Ok(TaskRunOutput::Folder(PathBuf::from(format!(
            "{}-{}",
            task_type.to_string(),
            task_run_record.id()
        ))))
    }

    async fn run_sum(
        &self,
        file_info: &crate::FileInfo,
        ctx: &ContentBaseCtx,
        task_run_record: &TaskRunRecord,
    ) -> anyhow::Result<()> {
        let chunks = self.chunk_task().chunk_content(file_info, ctx).await?;
        let llm = ctx.llm()?.0;

        for i in 0..chunks.len() {
            let chunk = &chunks[i];

            let previous_content = {
                if i == 0 {
                    "None".to_string()
                } else {
                    chunks[i - 1].clone()
                }
            };
            let user_prompt = format!(
                "Previous content:\n{}\n\nCurrent text:\n{}",
                previous_content, &chunk
            );

            let mut response = llm
                .process_single((
                    vec![
                        LLMMessage::new_system(format!(r#"You are an assistant skilled in document summarization.
You should try to summarize user input's document into a very short sentence.

Guidelines:
- Focus on essential information: Prioritize the text's core messages.
- Maintain clarity and conciseness: Craft your summary using accessible language.
- Capture the essence of the text.

Input:
- (optional) what is talking about in the previous document
- a piece of text that need to be summarized

Input Example:
```
Previous content:
xxx

Current text:
xxx
```

Additional Rules:
- Content: just response with the short sentence only, do not start with hint or prompt, do not contain anything else, e.g., "AI is changing the world."
- Focus: do not summarize the content in the previous text, focus on current piece of text
- Word count: aim for a summarization with no more than 30 words.
- Language: summarization should be in the same language with input"#).as_str()),
                        LLMMessage::new_user(&user_prompt),
                    ],
                    LLMInferenceParams::default(),
                ))
                .await?;

            let summarization = response.to_string().await?;

            let output_dir = task_run_record
                .output_path(&file_info.file_identifier, ctx)
                .await?;
            let output_path = output_dir.join(format!("{}.{}", i, "json"));

            self.write(
                output_path,
                json!({
                    "summarization": summarization
                })
                .to_string()
                .into(),
            )
            .await?;
        }

        Ok(())
    }

    fn sum_parameters(&self, ctx: &ContentBaseCtx) -> Value {
        json!({
            "model": ctx.llm().expect("llm is set").1
        })
    }

    async fn sum_content(
        &self,
        file_info: &crate::FileInfo,
        ctx: &ContentBaseCtx,
        index: usize,
    ) -> anyhow::Result<String> {
        let task_type: ContentTaskType = self.clone().into();
        let output_path = task_type
            .task_output_path(file_info, ctx)
            .await?
            .join(format!("{}.json", index));

        let content_str = self.read_to_string(output_path)?;
        let json_string: Value = serde_json::from_str(&content_str)?;
        let summarization = json_string["summarization"]
            .as_str()
            .ok_or(anyhow::anyhow!(
                "no summarization found in transcript chunk summarization file"
            ))?;
        Ok(summarization.to_string())
    }
}

#[derive(Clone, Debug, Default, Storage)]
pub struct RawTextChunkSumTask;

impl DocumentChunkSumTrait for RawTextChunkSumTask {
    fn chunk_task(&self) -> impl DocumentChunkTrait {
        RawTextChunkTask
    }
}

#[async_trait]
impl ContentTask for RawTextChunkSumTask {
    async fn task_output(&self, task_run_record: &TaskRunRecord) -> anyhow::Result<TaskRunOutput> {
        self.chunk_sum_output(task_run_record).await
    }

    async fn inner_run(
        &self,
        file_info: &crate::FileInfo,
        ctx: &ContentBaseCtx,
        task_run_record: &mut TaskRunRecord,
    ) -> anyhow::Result<()> {
        self.run_sum(file_info, ctx, task_run_record).await
    }

    fn task_parameters(&self, ctx: &ContentBaseCtx) -> Value {
        self.sum_parameters(ctx)
    }

    fn task_dependencies(&self) -> Vec<ContentTaskType> {
        vec![RawTextChunkTask.into()]
    }
}

impl Into<ContentTaskType> for RawTextChunkSumTask {
    fn into(self) -> ContentTaskType {
        ContentTaskType::RawText(RawTextTaskType::ChunkSum(self.clone()))
    }
}
