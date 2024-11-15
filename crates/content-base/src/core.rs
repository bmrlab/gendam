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
        // frame_description::VideoFrameDescriptionTask,
        frame_desc_embed::VideoFrameDescEmbedTask,
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

    /// 列出每种类型的内容处理需要执行的所有任务，因为有任务依赖关系，只需要列出最顶层的任务
    pub fn get_content_processing_tasks(
        metadata: &ContentMetadata,
    ) -> Vec<(ContentTaskType, TaskPriority)> {
        let mut tasks = vec![];

        // TODO: 现在好像不支持有些是 Low 有些是 Normal，会导致 Low 的任务被 cancel 并且 Normal 的任务也不被执行...
        // 比如先开始了 Video，优先级是 Low，然后开始了 Image 任务，优先级是 Normal，这时候 Video 任务会被 cancel，但是 Image 任务不会被执行
        match metadata {
            ContentMetadata::Video(metadata) => {
                if metadata.audio.is_some() {
                    tasks.push((VideoTransChunkSumEmbedTask.into(), TaskPriority::Normal));
                }
                // tasks.push((VideoFrameTask.into(), TaskPriority::Low));
                tasks.push((VideoFrameEmbeddingTask.into(), TaskPriority::Normal));
                tasks.push((VideoFrameDescEmbedTask.into(), TaskPriority::Normal));
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
