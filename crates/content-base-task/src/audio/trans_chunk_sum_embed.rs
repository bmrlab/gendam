use super::{
    trans_chunk::{AudioTransChunkTask, AudioTranscriptChunkTrait},
    trans_chunk_sum::{AudioTransChunkSumTask, AudioTransChunkSumTrait},
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
use std::path::PathBuf;
use storage_macro::Storage;

#[async_trait]
pub trait AudioTransChunkSumEmbedTrait: Into<ContentTaskType> + Storage + Clone {
    fn chunk_task(&self) -> impl AudioTranscriptChunkTrait;
    fn sum_task(&self) -> impl AudioTransChunkSumTrait;

    async fn run_sum_embed(
        &self,
        file_info: &crate::FileInfo,
        ctx: &ContentBaseCtx,
        task_run_record: &mut crate::record::TaskRunRecord,
    ) -> anyhow::Result<()> {
        let chunks = self
            .chunk_task()
            .chunk_content(&file_info.file_identifier, ctx)
            .await?;
        let llm = ctx.llm()?.0;
        for chunk in chunks.iter() {
            let summarization = self
                .sum_task()
                .sum_content(
                    &file_info.file_identifier,
                    ctx,
                    chunk.start_timestamp,
                    chunk.end_timestamp,
                )
                .await?;

            let user_prompt = format!(
                    "Please translate following content into English, and response with translation only, without anything else.\n{}",
                    summarization
                );
            let mut output = llm
                .process_single((
                    vec![LLMMessage::new_user(&user_prompt)],
                    LLMInferenceParams::default(),
                ))
                .await?;
            let summarization_en = output.to_string().await?;

            let output_dir = task_run_record
                .output_path(&file_info.file_identifier, ctx)
                .await?;
            let output_path = output_dir.join(format!(
                "{}-{}.{}",
                chunk.start_timestamp, chunk.end_timestamp, "json"
            ));

            ctx.save_text_embedding(&summarization_en, &output_path)
                .await?;
        }

        Ok(())
    }

    async fn embed_output(&self, task_run_record: &TaskRunRecord) -> anyhow::Result<TaskRunOutput> {
        let task_type: ContentTaskType = self.clone().into();
        Ok(TaskRunOutput::Folder(PathBuf::from(format!(
            "{}-{}",
            task_type.to_string(),
            task_run_record.id()
        ))))
    }

    fn embed_parameters(&self, ctx: &ContentBaseCtx) -> Value {
        json!({
            "model": ctx.text_embedding().expect("text embedding is set").1
        })
    }

    async fn embed_content(
        &self,
        file_identifier: &str,
        ctx: &ContentBaseCtx,
        start_timestamp: i64,
        end_timestamp: i64,
    ) -> anyhow::Result<Vec<f32>> {
        let task_type: ContentTaskType = self.clone().into();
        let output_path = task_type
            .task_output_path(file_identifier, ctx)
            .await?
            .join(format!("{}-{}.json", start_timestamp, end_timestamp,));
        let content_str = self.read_to_string(output_path)?;
        let embedding: Vec<f32> = serde_json::from_str(&content_str)?;

        Ok(embedding)
    }
}

#[derive(Clone, Debug, Storage, Default)]
pub struct AudioTransChunkSumEmbedTask;

impl AudioTransChunkSumEmbedTrait for AudioTransChunkSumEmbedTask {
    fn sum_task(&self) -> impl AudioTransChunkSumTrait {
        AudioTransChunkSumTask
    }

    fn chunk_task(&self) -> impl AudioTranscriptChunkTrait {
        AudioTransChunkTask
    }
}

#[async_trait]
impl ContentTask for AudioTransChunkSumEmbedTask {
    async fn task_output(&self, task_run_record: &TaskRunRecord) -> anyhow::Result<TaskRunOutput> {
        self.embed_output(task_run_record).await
    }

    async fn inner_run(
        &self,
        file_info: &crate::FileInfo,
        ctx: &ContentBaseCtx,
        task_run_record: &mut TaskRunRecord,
    ) -> anyhow::Result<()> {
        self.run_sum_embed(file_info, ctx, task_run_record).await
    }

    fn task_parameters(&self, ctx: &ContentBaseCtx) -> Value {
        self.embed_parameters(ctx)
    }

    fn task_dependencies(&self) -> Vec<ContentTaskType> {
        vec![AudioTransChunkSumTask.into()]
    }
}

impl Into<ContentTaskType> for AudioTransChunkSumEmbedTask {
    fn into(self) -> ContentTaskType {
        ContentTaskType::Audio(AudioTaskType::TransChunkSumEmbed(self.clone()))
    }
}
