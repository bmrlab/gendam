use super::{transcript::AudioTranscriptTask, AudioTaskType};
use crate::{
    record::{TaskRunOutput, TaskRunRecord},
    ContentTask, ContentTaskType, FileInfo,
};
use ai::{AudioTranscriptOutput, Transcription};
use async_trait::async_trait;
use content_base_context::ContentBaseCtx;
use content_handler::doc::chunk::naive_chunk;
use serde_json::{json, Value};
use std::path::PathBuf;
use storage_macro::Storage;

#[async_trait]
pub trait AudioTranscriptChunkTrait: Into<ContentTaskType> + Clone + Storage {
    fn transcript_task(&self) -> impl ContentTask;

    async fn chunk_output(
        &self,
        task_run_record: &TaskRunRecord,
    ) -> anyhow::Result<TaskRunOutput> {
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
        task_run_record: &mut TaskRunRecord,
    ) -> anyhow::Result<()> {
        let tokenizer = ctx.llm_tokenizer()?;

        let transcript_path = self
            .transcript_task()
            .task_output_path(file_info, ctx)
            .await?;
        let transcript_str = self.read_to_string(transcript_path)?;
        let transcript: AudioTranscriptOutput = serde_json::from_str(&transcript_str)?;

        let items = transcript
            .transcriptions
            .iter()
            .map(|v| TranscriptToChunk {
                start_timestamp: v.start_timestamp,
                end_timestamp: v.end_timestamp,
                text: v.text.clone(),
            })
            .collect::<Vec<_>>();

        let chunks = naive_chunk(&items, tokenizer, 100)?;

        // merge chunks
        let chunks = chunks
            .into_iter()
            .map(|v| Transcription {
                start_timestamp: v.first().expect("chunk not empty").start_timestamp,
                end_timestamp: v.last().expect("chunk not empty").end_timestamp,
                text: v
                    .iter()
                    .map(|v| v.text.clone())
                    .collect::<Vec<_>>()
                    .join("\n"),
            })
            .collect::<Vec<_>>();

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
    ) -> anyhow::Result<Vec<Transcription>> {
        let task_type: ContentTaskType = self.clone().into();
        let output_path = task_type.task_output_path(file_info, ctx).await?;
        let content = self.read_to_string(output_path)?;
        Ok(serde_json::from_str(&content)?)
    }
}

#[derive(Clone, Storage, Debug, Default)]
pub struct AudioTransChunkTask;

#[async_trait]
impl AudioTranscriptChunkTrait for AudioTransChunkTask {
    fn transcript_task(&self) -> impl ContentTask {
        AudioTranscriptTask
    }
}

#[derive(Clone)]
struct TranscriptToChunk {
    start_timestamp: i64,
    end_timestamp: i64,
    text: String,
}

impl ToString for TranscriptToChunk {
    fn to_string(&self) -> String {
        self.text.clone()
    }
}

#[async_trait]
impl ContentTask for AudioTransChunkTask {
    async fn task_output(&self, task_run_record: &TaskRunRecord) -> anyhow::Result<TaskRunOutput> {
        self.chunk_output(task_run_record).await
    }

    async fn inner_run(
        &self,
        file_info: &crate::FileInfo,
        ctx: &ContentBaseCtx,
        task_run_record: &mut TaskRunRecord,
    ) -> anyhow::Result<()> {
        self.run_chunk(file_info, ctx, task_run_record)
            .await
    }

    fn task_parameters(&self, ctx: &ContentBaseCtx) -> Value {
        self.chunk_parameters(ctx)
    }

    fn task_dependencies(&self) -> Vec<ContentTaskType> {
        vec![AudioTranscriptTask.into()]
    }
}

impl Into<ContentTaskType> for AudioTransChunkTask {
    fn into(self) -> ContentTaskType {
        ContentTaskType::Audio(AudioTaskType::TransChunk(self))
    }
}
