use crate::db::DB;
use crate::ContentBase;
use content_base_context::ContentBaseCtx;
use content_base_pool::{TaskPool, TaskPriority};
use content_base_task::{
    audio::{trans_chunk_sum_embed::AudioTransChunkSumEmbedTask, waveform::AudioWaveformTask},
    image::{desc_embed::ImageDescEmbedTask, embedding::ImageEmbeddingTask},
    raw_text::chunk_sum_embed::RawTextChunkSumEmbedTask,
    video::{frame::VideoFrameTask, trans_chunk_sum_embed::VideoTransChunkSumEmbedTask},
    web_page::chunk_sum_embed::WebPageChunkSumEmbedTask,
    ContentTaskType,
};
use content_metadata::ContentMetadata;
use qdrant_client::Qdrant;
use std::sync::Arc;
use tokio::sync::RwLock;

impl ContentBase {
    /// Create a new ContentBase with Context. The context will be cloned,
    /// so if need to modify context, a new ContentBase should be created.
    pub fn new(
        ctx: &ContentBaseCtx,
        qdrant_client: Arc<Qdrant>,
        db: Arc<RwLock<DB>>,
        language_collection_name: &str,
        vision_collection_name: &str,
    ) -> anyhow::Result<Self> {
        let task_pool = TaskPool::new(ctx, None)?;
        Ok(Self {
            ctx: ctx.clone(),
            task_pool,
            qdrant: qdrant_client,
            db,
            language_collection_name: language_collection_name.to_string(),
            vision_collection_name: vision_collection_name.to_string(),
        })
    }

    pub fn ctx(&self) -> &ContentBaseCtx {
        &self.ctx
    }

    pub fn tasks(metadata: &ContentMetadata) -> Vec<(ContentTaskType, TaskPriority)> {
        let mut tasks = vec![];

        match metadata {
            ContentMetadata::Video(metadata) => {
                tasks.push((VideoFrameTask.into(), TaskPriority::Low));
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
