use crate::db::{
    model::{
        audio::{AudioFrameModel, AudioModel},
        document::DocumentModel,
        image::ImageModel,
        text::TextModel,
        video::{ImageFrameModel, VideoModel},
        web::WebPageModel,
        PageModel,
    },
    DB,
};
use crate::{collect_async_results, ContentBase};
use content_base_context::ContentBaseCtx;
use content_base_pool::{TaskNotification, TaskPool, TaskPriority, TaskStatus};
use content_base_task::{
    audio::{
        trans_chunk::{AudioTransChunkTask, AudioTranscriptChunkTrait},
        trans_chunk_sum_embed::{AudioTransChunkSumEmbedTask, AudioTransChunkSumEmbedTrait},
        AudioTaskType,
    },
    image::{
        desc_embed::ImageDescEmbedTask, description::ImageDescriptionTask,
        embedding::ImageEmbeddingTask, ImageTaskType,
    },
    raw_text::{
        chunk::{DocumentChunkTrait, RawTextChunkTask},
        chunk_sum_embed::DocumentChunkSumEmbedTrait,
        RawTextTaskType,
    },
    video::{
        frame::{VideoFrameTask, VIDEO_FRAME_SUMMARY_BATCH_SIZE},
        frame_desc_embed::VideoFrameDescEmbedTask,
        frame_description::VideoFrameDescriptionTask,
        frame_embedding::VideoFrameEmbeddingTask,
        trans_chunk::VideoTransChunkTask,
        trans_chunk_sum_embed::VideoTransChunkSumEmbedTask,
        VideoTaskType,
    },
    web_page::{chunk::WebPageChunkTask, WebPageTaskType},
    ContentTaskType, FileInfo, TaskRecord,
};
use content_metadata::ContentMetadata;
use futures_util::future::try_join_all;
use serde::{Deserialize, Serialize};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::sync::mpsc::{self, Receiver};
use tokio::sync::RwLock;
use tracing::{debug, warn};

#[derive(Serialize, Deserialize)]
pub struct UpsertPayload {
    file_identifier: String,
    file_full_path_on_disk: PathBuf,
    // file_extension: Option<String>,
    metadata: ContentMetadata,
}

impl UpsertPayload {
    pub fn new(
        file_identifier: &str,
        file_full_path_on_disk: impl AsRef<Path>,
        metadata: &ContentMetadata,
    ) -> Self {
        Self {
            file_identifier: file_identifier.to_string(),
            file_full_path_on_disk: file_full_path_on_disk.as_ref().to_path_buf(),
            // file_extension: None,
            metadata: metadata.clone(),
        }
    }

    // pub fn with_extension(mut self, file_extension: &str) -> Self {
    //     self.file_extension = Some(file_extension.to_string());
    //     self
    // }
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
            file_full_path_on_disk: payload.file_full_path_on_disk.clone(),
        };

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
        let db = self.db.clone();
        let file_identifier_clone = file_identifier.to_string();
        tokio::spawn(async move {
            while let Some(notification) = inner_rx.recv().await {
                let task_type = notification.task_type.clone();
                let task_status = notification.status.clone();
                // receive notification from content_base_pool and send to client
                let _ = notification_tx.send(notification).await;
                // 对完成的任务进行后处理
                if let TaskStatus::Finished = task_status {
                    let _ = task_post_process(&ctx, &file_identifier_clone, &task_type, db.clone())
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
            &file_info.file_full_path_on_disk,
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

macro_rules! chunk_to_page {
    ($file_identifier:expr, $ctx:expr, $task_type:expr, $chunks:expr) => {{
        collect_async_results!($chunks
            .into_iter()
            .enumerate()
            .map(|(i, chunk)| async move {
                let embedding = $task_type.embed_content($file_identifier, $ctx, i).await?;
                anyhow::Result::<PageModel>::Ok(PageModel {
                    id: None,
                    text: vec![TextModel {
                        id: None,
                        data: chunk.clone(),
                        vector: embedding.clone(),
                        en_data: "".to_string(),
                        en_vector: vec![],
                    }],
                    image: vec![],
                    start_index: i as i32,
                    end_index: i as i32,
                })
            })
            .collect::<Vec<_>>())
    }};
}

#[tracing::instrument(level = "info", skip(ctx, db))]
async fn task_post_process(
    ctx: &ContentBaseCtx,
    file_identifier: &str,
    task_type: &ContentTaskType,
    db: Arc<RwLock<DB>>,
) -> anyhow::Result<()> {
    match task_type {
        ContentTaskType::Video(
            VideoTaskType::TransChunkSumEmbed(_)
            | VideoTaskType::FrameEmbedding(_)
            | VideoTaskType::FrameDescEmbed(_),
        ) => {
            // TransChunkSumEmbed, FrameDescEmbed 和 FrameEmbedding 结束后都触发 upsert_video_index_to_surrealdb
            // 如果有一个任务没完成，upsert_video_index_to_surrealdb 会报错
            upsert_video_index_to_surrealdb(ctx, file_identifier, db).await.map_err(|e| {
                tracing::warn!("either TransChunkSumEmbed or FrameEmbedding or FrameDescEmbed task not finished yet: {:?}", e);
                e
            })?;
            tracing::info!("video index upserted to surrealdb");
        }
        ContentTaskType::Audio(AudioTaskType::TransChunkSumEmbed(_)) => {
            let chunks = AudioTransChunkTask
                .chunk_content(file_identifier, ctx)
                .await?;
            let future = chunks
                .into_iter()
                .map(|chunk| async move {
                    let embedding = AudioTransChunkSumEmbedTask
                        .embed_content(
                            file_identifier,
                            ctx,
                            chunk.start_timestamp,
                            chunk.end_timestamp,
                        )
                        .await?;
                    Result::<AudioFrameModel, anyhow::Error>::Ok(AudioFrameModel {
                        id: None,
                        data: vec![TextModel {
                            id: None,
                            data: chunk.text.clone(),
                            vector: embedding.clone(),
                            // TODO: 是否需要英文
                            en_data: "".to_string(),
                            en_vector: vec![],
                        }],
                        start_timestamp: chunk.start_timestamp as f32,
                        end_timestamp: chunk.end_timestamp as f32,
                    })
                })
                .collect::<Vec<_>>();
            let audio_frame: anyhow::Result<Vec<AudioFrameModel>> = collect_async_results!(future);
            db.try_read()?
                .insert_audio(
                    AudioModel {
                        id: None,
                        audio_frame: audio_frame?,
                    },
                    file_identifier.to_string(),
                )
                .await?;
        }
        ContentTaskType::Image(ImageTaskType::DescEmbed(_) | ImageTaskType::Embedding(_)) => {
            // DescEmbed 和 Embedding 结束后都触发 upsert_image_index_to_surrealdb
            // 如果有一个任务没完成，upsert_image_index_to_surrealdb 会报错
            upsert_image_index_to_surrealdb(ctx, file_identifier, db).await.map_err(|e| {
                tracing::warn!("either image embedding or description embedding task not finished yet: {:?}", e);
                e
            })?;
            tracing::info!("image index upserted to surrealdb");
        }
        ContentTaskType::RawText(RawTextTaskType::ChunkSumEmbed(task_type)) => {
            let pages: anyhow::Result<Vec<PageModel>> = chunk_to_page!(
                file_identifier,
                ctx,
                task_type,
                RawTextChunkTask.chunk_content(file_identifier, ctx).await?
            );
            debug!("pages: {pages:?}");
            db.try_read()?
                .insert_document(DocumentModel::new(pages?), file_identifier.to_string())
                .await?;
        }
        ContentTaskType::WebPage(WebPageTaskType::ChunkSumEmbed(task_type)) => {
            let pages: anyhow::Result<Vec<PageModel>> = chunk_to_page!(
                file_identifier,
                ctx,
                task_type,
                WebPageChunkTask.chunk_content(file_identifier, ctx).await?
            );
            debug!("pages: {pages:?}");
            db.try_read()?
                .insert_web_page(WebPageModel::new(pages?), file_identifier.to_string())
                .await?;
        }
        _ => {}
    }
    Ok(())
}

#[tracing::instrument(skip_all)]
async fn upsert_video_index_to_surrealdb(
    ctx: &ContentBaseCtx,
    file_identifier: &str,
    db: Arc<RwLock<DB>>,
) -> anyhow::Result<()> {
    let chunks = VideoTransChunkTask
        .chunk_content(file_identifier, ctx)
        .await?;
    tracing::debug!("video chunks: {chunks:?}");
    let future = chunks
        .into_iter()
        .map(|chunk| async move {
            let embedding = VideoTransChunkSumEmbedTask
                .embed_content(
                    file_identifier,
                    ctx,
                    chunk.start_timestamp,
                    chunk.end_timestamp,
                )
                .await?;
            tracing::debug!("chunk: {chunk:?}, embedding: {:?}", embedding.len());
            Result::<AudioFrameModel, anyhow::Error>::Ok(AudioFrameModel {
                id: None,
                data: vec![TextModel {
                    id: None,
                    data: chunk.text.clone(),
                    vector: embedding.clone(),
                    // TODO: 是否需要英文
                    en_data: "".to_string(),
                    en_vector: vec![],
                }],
                start_timestamp: chunk.start_timestamp as f32,
                end_timestamp: chunk.end_timestamp as f32,
            })
        })
        .collect::<Vec<_>>();
    let audio_frame: Vec<AudioFrameModel> = try_join_all(future).await?;
    let frames = VideoFrameTask.frame_content(file_identifier, ctx).await?;
    tracing::debug!("video frames: {frames:?}");
    let future = frames
        .chunks(VIDEO_FRAME_SUMMARY_BATCH_SIZE)
        .into_iter()
        .map(|frame_infos_chunk| async move {
            let first_frame = frame_infos_chunk.first().expect("first chunk should exist");
            let last_frame = frame_infos_chunk.last().expect("last chunk should exist");

            let desc_embedding = VideoFrameDescEmbedTask
                .frame_desc_embed_content(
                    file_identifier,
                    ctx,
                    first_frame.timestamp,
                    last_frame.timestamp,
                )
                .await?;
            let description = VideoFrameDescriptionTask
                .frame_description_content(
                    file_identifier,
                    ctx,
                    first_frame.timestamp,
                    last_frame.timestamp,
                )
                .await?;
            // TODO: 优化?, 目前 embedding 只取第一个 chunk 的，description 取的是一个片段的
            let embedding = VideoFrameEmbeddingTask
                .frame_embedding_content(file_identifier, ctx, first_frame.timestamp)
                .await?;
            Result::<ImageFrameModel, anyhow::Error>::Ok(ImageFrameModel {
                id: None,
                data: vec![ImageModel {
                    id: None,
                    prompt: description,
                    vector: embedding,
                    prompt_vector: desc_embedding,
                }],
                start_timestamp: first_frame.timestamp as f32,
                end_timestamp: last_frame.timestamp as f32,
            })
        });
    let image_frame: Vec<ImageFrameModel> = try_join_all(future).await?;
    db.try_read()?
        .insert_video(
            VideoModel {
                id: None,
                audio_frame,
                image_frame,
            },
            file_identifier.to_string(),
        )
        .await?;
    Ok(())
}

#[tracing::instrument(skip_all)]
async fn upsert_image_index_to_surrealdb(
    ctx: &ContentBaseCtx,
    file_identifier: &str,
    db: Arc<RwLock<DB>>,
) -> anyhow::Result<()> {
    // 不用 task_type.desc_embed_content，用 ImageDescEmbedTask 创建个空实例，统一写法
    let desc_embedding = ImageDescEmbedTask
        .desc_embed_content(file_identifier, ctx)
        .await?;
    let description = ImageDescriptionTask
        .description_content(file_identifier, ctx)
        .await?;
    let embedding = ImageEmbeddingTask
        .embedding_content(file_identifier, ctx)
        .await?;
    db.try_read()?
        .insert_image(
            ImageModel {
                id: None,
                prompt: description,
                vector: embedding,
                prompt_vector: desc_embedding,
            },
            Some(file_identifier.to_string()),
        )
        .await?;
    Ok(())
}

// #[tracing::instrument(skip_all)]
// async fn transcript_sum_embed_post_process<T, TFn>(
//     ctx: &ContentBaseCtx,
//     qdrant: Arc<Qdrant>,
//     collection_name: &str,
//     file_info: &FileInfo,
//     chunk_task: impl AudioTranscriptChunkTrait,
//     embed_task: impl AudioTransChunkSumEmbedTrait + ContentTask,
//     fn_search_metadata: TFn,
// ) -> anyhow::Result<()>
// where
//     T: Into<SearchMetadata>,
//     TFn: Fn(i64, i64) -> T,
// {
//     let chunks = chunk_task.chunk_content(file_info, ctx).await?;
//
//     for chunk in chunks.iter() {
//         let metadata = fn_search_metadata(chunk.start_timestamp, chunk.end_timestamp);
//         let payload = SearchPayload {
//             file_identifier: file_info.file_identifier.clone(),
//             task_type: embed_task.clone().into(),
//             metadata: metadata.into(),
//         };
//         let embedding = embed_task
//             .embed_content(file_info, ctx, chunk.start_timestamp, chunk.end_timestamp)
//             .await?;
//
//         let point = PointStruct::new(payload.uuid().to_string(), embedding, payload);
//
//         // TODO 这里其实可以直接用 upsert_points_chunked，但是似乎有点问题，后续再优化下
//         if let Err(e) = qdrant
//             .upsert_points(UpsertPointsBuilder::new(collection_name, vec![point]).wait(true))
//             .await
//         {
//             warn!("failed to upsert points: {e:?}");
//         }
//     }
//
//     Ok(())
// }
