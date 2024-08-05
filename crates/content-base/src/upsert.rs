use crate::{
    query::payload::{
        audio::AudioSearchMetadata, video::VideoSearchMetadata, SearchMetadata, SearchPayload,
    },
    ContentBase,
};
use content_base_context::ContentBaseCtx;
use content_base_pool::{TaskNotification, TaskPool, TaskPriority};
use content_base_task::{
    audio::{
        trans_chunk::{AudioTransChunkTask, AudioTranscriptChunkTrait},
        trans_chunk_sum_embed::{AudioTransChunkSumEmbedTask, AudioTransChunkSumEmbedTrait},
        waveform::AudioWaveformTask,
        AudioTaskType,
    },
    video::{
        frame::VideoFrameTask, trans_chunk::VideoTransChunkTask,
        trans_chunk_sum_embed::VideoTransChunkSumEmbedTask, VideoTaskType,
    },
    ContentTask, ContentTaskType, FileInfo, TaskRecord,
};
use content_handler::file_metadata;
use content_metadata::ContentMetadata;
use qdrant_client::qdrant::{PointStruct, UpsertPointsBuilder};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::sync::mpsc::{self, Receiver};
use tracing::warn;

#[derive(Serialize, Deserialize)]
pub struct UpsertPayload {
    file_identifier: String,
    file_path: PathBuf,
    metadata: Option<ContentMetadata>,
}

impl UpsertPayload {
    pub fn new(file_identifier: &str, file_path: impl AsRef<Path>) -> Self {
        Self {
            file_identifier: file_identifier.to_string(),
            file_path: file_path.as_ref().to_path_buf(),
            metadata: None,
        }
    }

    pub fn with_metadata(mut self, metadata: ContentMetadata) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

impl ContentBase {
    pub async fn upsert(
        &self,
        payload: UpsertPayload,
    ) -> anyhow::Result<Receiver<TaskNotification>> {
        let task_pool = self.task_pool.clone();
        let file_identifier = &payload.file_identifier.clone();

        let mut task_record = TaskRecord::from_content_base(file_identifier, &self.ctx).await;
        let metadata = match payload.metadata.clone() {
            Some(metadata) => metadata,
            _ => match task_record.metadata() {
                ContentMetadata::Unknown => {
                    file_metadata(&payload.file_path).expect("got file metadata")
                }
                _ => task_record.metadata().clone(),
            },
        };

        let (notification_tx, notification_rx) = mpsc::channel(512);
        let (inner_tx, mut inner_rx) = mpsc::channel(512);

        if let Err(e) = task_record.set_metadata(&self.ctx, &metadata).await {
            warn!("failed to set metadata: {e:?}");
        }

        let file_info = FileInfo {
            file_identifier: payload.file_identifier.clone(),
            file_path: payload.file_path.clone(),
        };
        let file_info_clone = file_info.clone();

        tokio::spawn(async move {
            match metadata {
                ContentMetadata::Video(metadata) => {
                    run_task(
                        &task_pool,
                        &file_info,
                        VideoFrameTask,
                        Some(TaskPriority::Low),
                        Some(inner_tx.clone()),
                    )
                    .await;

                    if metadata.audio.is_some() {
                        run_task(
                            &task_pool,
                            &file_info,
                            VideoTransChunkSumEmbedTask,
                            Some(TaskPriority::Low),
                            Some(inner_tx.clone()),
                        )
                        .await;
                    }
                }
                ContentMetadata::Audio(_metadata) => {
                    run_task(
                        &task_pool,
                        &file_info,
                        AudioWaveformTask,
                        Some(TaskPriority::Normal),
                        Some(inner_tx.clone()),
                    )
                    .await;
                    run_task(
                        &task_pool,
                        &file_info,
                        AudioTransChunkSumEmbedTask,
                        Some(TaskPriority::Normal),
                        Some(inner_tx.clone()),
                    )
                    .await;
                }
                ContentMetadata::Unknown => {
                    warn!(
                        "unknown metadata for {}, do not trigger any tasks",
                        &payload.file_identifier
                    );
                }
            }
        });

        // 对 task notification 做进一步处理
        let ctx = self.ctx.clone();
        tokio::spawn(async move {
            while let Some(notification) = inner_rx.recv().await {
                let task_type = notification.task_type.clone();
                let _ = notification_tx.send(notification).await;

                // 对完成的任务进行后处理
                task_post_process(&ctx, &file_info_clone, &task_type).await;
            }
        });

        Ok(notification_rx)
    }
}

async fn run_task(
    task_pool: &TaskPool,
    file_info: &FileInfo,
    task_type: impl Into<ContentTaskType>,
    priority: Option<TaskPriority>,
    notification_tx: Option<mpsc::Sender<TaskNotification>>,
) {
    let task_type: ContentTaskType = task_type.into();
    if let Err(e) = task_pool
        .add_task(
            &file_info.file_identifier,
            &file_info.file_path,
            &task_type,
            priority,
            notification_tx,
        )
        .await
    {
        warn!(
            "failed to add task {}{}: {}",
            &file_info.file_identifier, &task_type, e
        );
    }
}

async fn task_post_process(
    ctx: &ContentBaseCtx,
    file_info: &FileInfo,
    task_type: &ContentTaskType,
) {
    match task_type {
        ContentTaskType::Video(VideoTaskType::TransChunkSumEmbed(_)) => {
            let _ = transcript_sum_embed_post_process(
                ctx,
                file_info,
                VideoTransChunkTask,
                VideoTransChunkSumEmbedTask,
                |start, end| VideoSearchMetadata::new(start, end),
            )
            .await;
        }
        ContentTaskType::Audio(AudioTaskType::TransChunkSumEmbed(_)) => {
            let _ = transcript_sum_embed_post_process(
                ctx,
                file_info,
                AudioTransChunkTask,
                AudioTransChunkSumEmbedTask,
                |start, end| AudioSearchMetadata::new(start, end),
            )
            .await;
        }
        _ => {}
    }
}

#[tracing::instrument(skip_all)]
async fn transcript_sum_embed_post_process<T, TFn>(
    ctx: &ContentBaseCtx,
    file_info: &FileInfo,
    chunk_task: impl AudioTranscriptChunkTrait,
    embed_task: impl AudioTransChunkSumEmbedTrait + ContentTask,
    fn_search_metadata: TFn,
) -> anyhow::Result<()>
where
    T: Into<SearchMetadata>,
    TFn: Fn(i64, i64) -> T,
{
    let qdrant = ctx.qdrant();
    let collection_name = ctx.language_collection_name();

    let chunks = chunk_task.chunk_content(file_info, ctx).await?;

    for chunk in chunks.iter() {
        let metadata = fn_search_metadata(chunk.start_timestamp, chunk.end_timestamp);
        let payload = SearchPayload {
            file_identifier: file_info.file_identifier.clone(),
            task_type: embed_task.clone().into(),
            metadata: metadata.into(),
        };
        let embedding = embed_task
            .embed_content(file_info, ctx, chunk.start_timestamp, chunk.end_timestamp)
            .await?;

        let point = PointStruct::new(payload.uuid().to_string(), embedding, payload);

        // TODO 这里其实可以直接用 upsert_points_chunked，但是似乎有点问题，后续再优化下
        if let Err(e) = qdrant
            .upsert_points(UpsertPointsBuilder::new(collection_name, vec![point]).wait(true))
            .await
        {
            warn!("failed to upsert points: {e:?}");
        }
    }

    Ok(())
}
