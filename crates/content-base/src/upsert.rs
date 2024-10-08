use crate::{
    query::payload::{
        audio::AudioIndexMetadata, image::ImageIndexMetadata, raw_text::RawTextIndexMetadata,
        video::VideoIndexMetadata, web_page::WebPageIndexMetadata, ContentIndexMetadata,
        ContentIndexPayload,
    },
    ContentBase,
};
use content_base_context::ContentBaseCtx;
use content_base_pool::{TaskNotification, TaskPool, TaskPriority, TaskStatus};
use content_base_task::{
    audio::{
        trans_chunk::{AudioTransChunkTask, AudioTranscriptChunkTrait},
        trans_chunk_sum_embed::{AudioTransChunkSumEmbedTask, AudioTransChunkSumEmbedTrait},
        AudioTaskType,
    },
    image::ImageTaskType,
    raw_text::{
        chunk::{DocumentChunkTrait, RawTextChunkTask},
        chunk_sum_embed::DocumentChunkSumEmbedTrait,
        RawTextTaskType,
    },
    video::{
        trans_chunk::VideoTransChunkTask, trans_chunk_sum_embed::VideoTransChunkSumEmbedTask,
        VideoTaskType,
    },
    web_page::{chunk::WebPageChunkTask, WebPageTaskType},
    ContentTask, ContentTaskType, FileInfo, TaskRecord,
};
use content_metadata::ContentMetadata;
use qdrant_client::{
    qdrant::{PointStruct, UpsertPointsBuilder},
    Qdrant,
};
use serde::{Deserialize, Serialize};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::sync::mpsc::{self, Receiver};
use tracing::warn;

#[derive(Serialize, Deserialize)]
pub struct UpsertPayload {
    file_identifier: String,
    file_path: PathBuf,
    file_extension: Option<String>,
    metadata: ContentMetadata,
}

impl UpsertPayload {
    pub fn new(
        file_identifier: &str,
        file_path: impl AsRef<Path>,
        metadata: &ContentMetadata,
    ) -> Self {
        Self {
            file_identifier: file_identifier.to_string(),
            file_path: file_path.as_ref().to_path_buf(),
            file_extension: None,
            metadata: metadata.clone(),
        }
    }

    pub fn with_extension(mut self, file_extension: &str) -> Self {
        self.file_extension = Some(file_extension.to_string());
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

        let (notification_tx, notification_rx) = mpsc::channel(512);
        let (inner_tx, mut inner_rx) = mpsc::channel(512);

        if let Err(e) = task_record.set_metadata(&self.ctx, &payload.metadata).await {
            warn!("failed to set metadata: {e:?}");
        }

        let file_info = FileInfo {
            file_identifier: payload.file_identifier.clone(),
            file_path: payload.file_path.clone(),
        };
        let file_info_clone = file_info.clone();

        let tasks = Self::tasks(&payload.metadata);

        tokio::spawn(async move {
            for (task, priority) in tasks {
                run_task(
                    &task_pool,
                    &file_info,
                    task,
                    Some(priority),
                    Some(inner_tx.clone()),
                )
                .await;
            }
        });

        // 对 task notification 做进一步处理
        let ctx = self.ctx.clone();
        let qdrant = self.qdrant.clone();
        let language_collection_name = self.language_collection_name.clone();
        let vision_collection_name = self.vision_collection_name.clone();
        tokio::spawn(async move {
            while let Some(notification) = inner_rx.recv().await {
                let task_type = notification.task_type.clone();
                let task_status = notification.status.clone();
                // receive notification from content_base_pool and send to client
                let _ = notification_tx.send(notification).await;
                // 对完成的任务进行后处理
                if let TaskStatus::Finished = task_status {
                    task_post_process(
                        &ctx,
                        &file_info_clone,
                        &task_type,
                        qdrant.clone(),
                        language_collection_name.as_str(),
                        vision_collection_name.as_str(),
                    )
                    .await;
                }
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
    qdrant: Arc<Qdrant>,
    language_collection_name: &str,
    _vision_collection_name: &str,
) {
    match task_type {
        ContentTaskType::Video(VideoTaskType::TransChunkSumEmbed(_)) => {
            let _ = transcript_sum_embed_post_process(
                ctx,
                qdrant,
                language_collection_name,
                file_info,
                VideoTransChunkTask,
                VideoTransChunkSumEmbedTask,
                |start, end| VideoIndexMetadata::new(start, end),
            )
            .await;
        }
        ContentTaskType::Audio(AudioTaskType::TransChunkSumEmbed(_)) => {
            let _ = transcript_sum_embed_post_process(
                ctx,
                qdrant,
                language_collection_name,
                file_info,
                AudioTransChunkTask,
                AudioTransChunkSumEmbedTask,
                |start, end| AudioIndexMetadata::new(start, end),
            )
            .await;
        }
        ContentTaskType::Image(ImageTaskType::DescEmbed(task_type)) => {
            if let Ok(embedding) = task_type.embed_content(file_info, ctx).await {
                let payload = ContentIndexPayload {
                    file_identifier: file_info.file_identifier.clone(),
                    task_type: task_type.clone().into(),
                    metadata: ContentIndexMetadata::Image(ImageIndexMetadata {}),
                };

                let point = PointStruct::new(payload.uuid().to_string(), embedding, payload);

                if let Err(e) = qdrant
                    .upsert_points(
                        UpsertPointsBuilder::new(language_collection_name, vec![point]).wait(true),
                    )
                    .await
                {
                    warn!("failed to upsert points: {e:?}");
                }
            }
        }
        ContentTaskType::RawText(RawTextTaskType::ChunkSumEmbed(task_type)) => {
            if let Ok(chunks) = RawTextChunkTask.chunk_content(file_info, ctx).await {
                for i in 0..chunks.len() {
                    if let Ok(embedding) = task_type.embed_content(file_info, ctx, i).await {
                        let payload = ContentIndexPayload {
                            file_identifier: file_info.file_identifier.clone(),
                            task_type: task_type.clone().into(),
                            metadata: RawTextIndexMetadata {
                                start_index: i,
                                end_index: i,
                            }
                            .into(),
                        };
                        let point =
                            PointStruct::new(payload.uuid().to_string(), embedding, payload);

                        if let Err(e) = qdrant
                            .upsert_points(
                                UpsertPointsBuilder::new(language_collection_name, vec![point])
                                    .wait(true),
                            )
                            .await
                        {
                            warn!("failed to upsert points: {e:?}");
                        }
                    }
                }
            }
        }
        ContentTaskType::WebPage(WebPageTaskType::ChunkSumEmbed(task_type)) => {
            if let Ok(chunks) = WebPageChunkTask.chunk_content(file_info, ctx).await {
                for i in 0..chunks.len() {
                    if let Ok(embedding) = task_type.embed_content(file_info, ctx, i).await {
                        let payload = ContentIndexPayload {
                            file_identifier: file_info.file_identifier.clone(),
                            task_type: task_type.clone().into(),
                            metadata: WebPageIndexMetadata {
                                start_index: i,
                                end_index: i,
                            }
                            .into(),
                        };
                        let point =
                            PointStruct::new(payload.uuid().to_string(), embedding, payload);

                        if let Err(e) = qdrant
                            .upsert_points(
                                UpsertPointsBuilder::new(language_collection_name, vec![point])
                                    .wait(true),
                            )
                            .await
                        {
                            warn!("failed to upsert points: {e:?}");
                        }
                    }
                }
            }
        }
        _ => {}
    }
}

#[tracing::instrument(skip_all)]
async fn transcript_sum_embed_post_process<T, TFn>(
    ctx: &ContentBaseCtx,
    qdrant: Arc<Qdrant>,
    collection_name: &str,
    file_info: &FileInfo,
    chunk_task: impl AudioTranscriptChunkTrait,
    embed_task: impl AudioTransChunkSumEmbedTrait + ContentTask,
    fn_search_metadata: TFn,
) -> anyhow::Result<()>
where
    T: Into<ContentIndexMetadata>,
    TFn: Fn(i64, i64) -> T,
{
    let chunks = chunk_task.chunk_content(file_info, ctx).await?;

    for chunk in chunks.iter() {
        let metadata = fn_search_metadata(chunk.start_timestamp, chunk.end_timestamp);
        let payload = ContentIndexPayload {
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
