use super::{
    trans_chunk::{AudioTransChunkTask, AudioTranscriptChunkTrait},
    trans_chunk_sum::{AudioTransChunkSumTrait, AudioTransChunkSumTask},
    AudioTaskType,
};
use crate::{
    record::{TaskRunOutput, TaskRunRecord},
    ContentTask, ContentTaskType,
};
use async_trait::async_trait;
use content_base_context::ContentBaseCtx;
use qdrant_client::qdrant::{PointStruct, UpsertPointsBuilder};
use serde_json::{json, Value};
use uuid::Uuid;
use std::{path::PathBuf};
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
        let qdrant = ctx.qdrant();
        let chunks = self.chunk_task().chunk_content(file_info, ctx).await?;
        let collection_name = ctx.language_collection_name();

        for chunk in chunks.iter() {
            let summarization = self
                .sum_task()
                .sum_content(file_info, ctx, chunk.start_timestamp, chunk.end_timestamp)
                .await?;

            let output_dir = task_run_record
                .output_path(&file_info.file_identifier, ctx)
                .await?;
            let output_path = output_dir.join(format!(
                "{}-{}.{}",
                chunk.start_timestamp, chunk.end_timestamp, "json"
            ));

            let embedding = ctx
                .save_text_embedding(&summarization, &output_path)
                .await?;

            // FIXME use correct payload and uuid
            // let payload = ContentPayload::TranscriptChunkSummarization {
            //     file_identifier: file_info.file_identifier.to_string(),
            //     start_timestamp: chunk.start_timestamp,
            //     end_timestamp: chunk.end_timestamp,
            // };
            let point = PointStruct::new(Uuid::new_v4().to_string(), embedding, serde_json::Map::new());
            if let Err(e) = qdrant
                .upsert_points(UpsertPointsBuilder::new(collection_name, vec![point]).wait(true))
                .await
            {
                tracing::debug!("qdrant upsert error: {:?}", e);
            }
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
