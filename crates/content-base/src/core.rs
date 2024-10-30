use crate::db::DB;
use crate::ContentBase;
use content_base_context::ContentBaseCtx;
use content_base_pool::{TaskPool, TaskPriority};
use content_base_task::{
    audio::{trans_chunk_sum_embed::AudioTransChunkSumEmbedTask, waveform::AudioWaveformTask},
    image::{desc_embed::ImageDescEmbedTask, embedding::ImageEmbeddingTask},
    raw_text::chunk_sum_embed::RawTextChunkSumEmbedTask,
    video::{
        // frame::VideoFrameTask,
        frame_description::VideoFrameDescriptionTask,
        frame_embedding::VideoFrameEmbeddingTask,
        trans_chunk_sum_embed::VideoTransChunkSumEmbedTask,
    },
    web_page::chunk_sum_embed::WebPageChunkSumEmbedTask,
    ContentTaskType,
};
use content_metadata::ContentMetadata;
use std::sync::Arc;
use tokio::sync::RwLock;

impl ContentBase {
    /// Create a new ContentBase with Context. The context will be cloned,
    /// so if need to modify context, a new ContentBase should be created.
    pub fn new(ctx: &ContentBaseCtx, db: Arc<RwLock<DB>>) -> anyhow::Result<Self> {
        let task_pool = TaskPool::new(ctx, None)?;
        Ok(Self {
            ctx: ctx.clone(),
            task_pool,
            db,
        })
    }

    pub fn ctx(&self) -> &ContentBaseCtx {
        &self.ctx
    }

    pub fn tasks(metadata: &ContentMetadata) -> Vec<(ContentTaskType, TaskPriority)> {
        let mut tasks = vec![];

        match metadata {
            ContentMetadata::Video(metadata) => {
                // tasks.push((VideoFrameTask.into(), TaskPriority::Low));
                tasks.push((VideoFrameEmbeddingTask.into(), TaskPriority::Low));
                tasks.push((VideoFrameDescriptionTask.into(), TaskPriority::Low));
                if metadata.audio.is_some() {
                    tasks.push((VideoTransChunkSumEmbedTask.into(), TaskPriority::Low));
                }
            }
            ContentMetadata::Audio(_metadata) => {
                tasks.extend([
                    (AudioWaveformTask.into(), TaskPriority::Normal),
                    (AudioTransChunkSumEmbedTask.into(), TaskPriority::Normal),
                ]);
            }
            ContentMetadata::Image(_) => {
                tasks.extend([
                    (ImageEmbeddingTask.into(), TaskPriority::Normal),
                    (ImageDescEmbedTask.into(), TaskPriority::Normal),
                ]);
            }
            ContentMetadata::RawText(_) => {
                tasks.push((RawTextChunkSumEmbedTask.into(), TaskPriority::Normal));
            }
            ContentMetadata::WebPage(_) => {
                tasks.push((WebPageChunkSumEmbedTask.into(), TaskPriority::Normal));
            }
            _ => {
                tracing::warn!("unsupported metadata, do not have any tasks");
            }
        }

        tasks
    }
}
