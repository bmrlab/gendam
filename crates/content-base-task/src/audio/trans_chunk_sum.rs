use std::path::PathBuf;

use super::{
    trans_chunk::{AudioTransChunkTask, AudioTranscriptChunkTrait},
    transcript::{AudioTranscriptTask, AudioTranscriptTrait},
    AudioTaskType,
};
use crate::{
    record::{TaskRunOutput, TaskRunRecord},
    ContentTask, ContentTaskType,
};
use ai::llm::{LLMInferenceParams, LLMMessage};
use async_trait::async_trait;
use content_base_context::ContentBaseCtx;
use serde_json::{json, Value};
use storage_macro::Storage;

#[async_trait]
pub trait AudioTransChunkSumTrait: Into<ContentTaskType> + Clone + Storage {
    fn transcript_task(&self) -> impl AudioTranscriptTrait;
    fn chunk_task(&self) -> impl AudioTranscriptChunkTrait;

    async fn run_sum(
        &self,
        file_info: &crate::FileInfo,
        ctx: &ContentBaseCtx,
        task_run_record: &mut crate::record::TaskRunRecord,
    ) -> anyhow::Result<()> {
        let transcript = self
            .transcript_task()
            .transcript_content(file_info, ctx)
            .await?;
        let chunks = self
            .chunk_task()
            .chunk_content(file_info, ctx)
            .await?;

        let llm = ctx.llm()?.0;

        for i in 0..chunks.len() {
            let chunk = &chunks[i];

            let previous_content = {
                if i == 0 {
                    "None".to_string()
                } else {
                    chunks[i - 1].text.clone()
                }
            };
            let user_prompt = format!(
                "Previous content:\n{}\n\nCurrent transcript:\n{}",
                previous_content, chunk.text
            );

            let mut response = llm
                .process_single((
                    vec![
                        LLMMessage::System(format!(r#"You are an assistant skilled in video transcript summarization.
You should try to summarize user input's transcript into a very short sentence.

Guidelines:
- Focus on essential information: Prioritize the transcript's core messages.
- Maintain clarity and conciseness: Craft your summary using accessible language.
- Capture the essence of the transcript.

Input:
- (optional) what is talking about in the previous video
- a piece of video transcript that need to be summarized

Input Example:
```
Previous content:
xxx

Current transcript:
xxx
```

Additional Rules:
- Content: just response with the short sentence only, do not start with hint or prompt, do not contain anything else, e.g., "The speaker is talking about his childhood."
- Focus: do not summarize the content in the previous video, focus on current piece of video transcript
- Word count: aim for a summarization with no more than 30 words.
- Language: summarization should be in the same language with input, which is {language}"#, language = transcript.language.as_ref())),
                        LLMMessage::User(user_prompt),
                    ],
                    LLMInferenceParams::default(),
                ))
                .await?;

            let summarization = response.to_string().await?;

            let output_dir = task_run_record
                .output_path(&file_info.file_identifier, ctx)
                .await?;
            let output_path = output_dir.join(format!(
                "{}-{}.{}",
                chunk.start_timestamp, chunk.end_timestamp, "json"
            ));

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

    async fn audio_trans_sum_output(
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

    fn sum_parameters(&self, ctx: &ContentBaseCtx) -> Value {
        json!({
            "model": ctx.llm().expect("llm is set").1
        })
    }

    async fn sum_content(
        &self,
        file_info: &crate::FileInfo,
        ctx: &ContentBaseCtx,
        start_timestamp: i64,
        end_timestamp: i64,
    ) -> anyhow::Result<String> {
        let task_type: ContentTaskType = self.clone().into();
        let output_path = task_type
            .task_output_path(file_info, ctx)
            .await?
            .join(format!("{}-{}.json", start_timestamp, end_timestamp,));
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

#[derive(Clone, Storage, Debug, Default)]
pub struct AudioTransChunkSumTask;

#[async_trait]
impl AudioTransChunkSumTrait for AudioTransChunkSumTask {
    fn chunk_task(&self) -> impl AudioTranscriptChunkTrait {
        AudioTransChunkTask
    }

    fn transcript_task(&self) -> impl AudioTranscriptTrait {
        AudioTranscriptTask
    }
}

#[async_trait]
impl ContentTask for AudioTransChunkSumTask {
    async fn task_output(&self, task_run_record: &TaskRunRecord) -> anyhow::Result<TaskRunOutput> {
        self.audio_trans_sum_output(task_run_record).await
    }

    async fn inner_run(
        &self,
        file_info: &crate::FileInfo,
        ctx: &ContentBaseCtx,
        task_run_record: &mut TaskRunRecord,
    ) -> anyhow::Result<()> {
        self.run_sum(file_info, ctx, task_run_record)
            .await
    }

    fn task_parameters(&self, ctx: &ContentBaseCtx) -> Value {
        self.sum_parameters(ctx)
    }

    fn task_dependencies(&self) -> Vec<ContentTaskType> {
        vec![AudioTransChunkTask.into()]
    }
}

impl Into<ContentTaskType> for AudioTransChunkSumTask {
    fn into(self) -> ContentTaskType {
        ContentTaskType::Audio(AudioTaskType::TransChunkSum(self.clone()))
    }
}
