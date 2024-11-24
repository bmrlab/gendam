use crate::db::{
    model::{
        audio::{AudioFrameModel, AudioModel},
        document::DocumentModel,
        image::ImageModel,
        page::PageModel,
        text::TextModel,
        video::{ImageFrameModel, VideoModel},
        web_page::WebPageModel,
    },
    DB,
};
use crate::{collect_async_results, ContentBase};
use content_base_context::ContentBaseCtx;
use content_base_pool::{TaskNotification, TaskStatus};
use content_base_task::{
    audio::{
        trans_chunk::{AudioTransChunkTask, AudioTranscriptChunkTrait},
        trans_chunk_sum::{AudioTransChunkSumTask, AudioTransChunkSumTrait},
        trans_chunk_sum_embed::{AudioTransChunkSumEmbedTask, AudioTransChunkSumEmbedTrait},
    },
    image::{
        desc_embed::ImageDescEmbedTask, description::ImageDescriptionTask,
        embedding::ImageEmbeddingTask,
    },
    raw_text::{
        chunk::{DocumentChunkTrait, RawTextChunkTask},
        chunk_sum_embed::{DocumentChunkSumEmbedTrait, RawTextChunkSumEmbedTask},
    },
    video::{
        frame::{VideoFrameTask, VIDEO_FRAME_SUMMARY_BATCH_SIZE},
        frame_desc_embed::VideoFrameDescEmbedTask,
        frame_description::VideoFrameDescriptionTask,
        frame_embedding::VideoFrameEmbeddingTask,
        trans_chunk::VideoTransChunkTask,
        trans_chunk_sum::VideoTransChunkSumTask,
        trans_chunk_sum_embed::VideoTransChunkSumEmbedTask,
    },
    web_page::{chunk::WebPageChunkTask, chunk_sum_embed::WebPageChunkSumEmbedTask},
    FileInfo, TaskRecord,
};
use content_metadata::{
    audio::AudioMetadata, image::ImageMetadata, raw_text::RawTextMetadata, video::VideoMetadata,
    web_page::WebPageMetadata, ContentMetadata, ContentType,
};
use futures_util::future::try_join_all;
use serde::{Deserialize, Serialize};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::sync::mpsc::{self, Receiver};
use tokio::sync::RwLock;
use tracing::Instrument;

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
    #[tracing::instrument(skip_all, fields(
        hash = %payload.file_identifier,
        content_type = %ContentType::from(&payload.metadata)
    ))]
    pub async fn upsert(
        &self,
        payload: UpsertPayload,
    ) -> anyhow::Result<Receiver<TaskNotification>> {
        let task_pool = self.task_pool.clone();

        let mut task_record =
            TaskRecord::from_content_base(payload.file_identifier.as_str(), &self.ctx).await;

        let (notification_tx, notification_rx) = mpsc::channel(512);
        let (inner_tx, mut inner_rx) = mpsc::channel(512);

        if let Err(e) = task_record.set_metadata(&self.ctx, &payload.metadata).await {
            tracing::warn!("failed to set metadata: {e:?}");
        }

        let file_info = FileInfo {
            file_identifier: payload.file_identifier.clone(),
            file_full_path_on_disk: payload.file_full_path_on_disk.clone(),
        };

        let tasks = Self::get_content_processing_tasks(&payload.metadata);
        let mut unfinished_tasks = std::collections::HashSet::new();
        for (task_type, _) in tasks.iter() {
            // ContentTaskType 实现了 to_string 和 Eq, 可以 clone 了以后用于 HashSet
            // see crates/content-base-task/src/task.rs
            unfinished_tasks.insert(task_type.clone());
        }

        // 内容被处理的入口
        tokio::spawn({
            let file_identifier = file_info.file_identifier.clone();
            let file_path = file_info.file_full_path_on_disk.clone();
            let inner_tx = inner_tx.clone();
            async move {
                for (task, priority) in tasks {
                    let priority = Some(priority);
                    // 当所有 inner_tx 的 clone 都被 drop 后通道才会被关闭然后 inner_rx 被 drop
                    let notify_tx: Option<mpsc::Sender<TaskNotification>> = Some(inner_tx.clone());
                    match task_pool
                        .add_task(&file_identifier, &file_path, &task, priority, notify_tx)
                        .await
                    {
                        Err(e) => {
                            tracing::error!(error=?e, task_type=%task, "Failed to add task");
                        }
                        Ok(()) => {
                            tracing::info!(task_type=%task, "Task added to TaskPool");
                        }
                    };
                }
            }
            .instrument(tracing::Span::current())
        });

        tokio::spawn({
            let ctx = self.ctx.clone();
            let surrealdb_client = self.surrealdb_client.clone();
            let file_identifier = file_info.file_identifier.to_string();
            // 对 task notification 做进一步处理
            async move {
                while let Some(notification) = inner_rx.recv().await {
                    let task_type = notification.task_type.clone();
                    let task_status = notification.status.clone();
                    // receive notification from content_base_pool and send to client
                    let _ = notification_tx.send(notification).await;
                    // 对完成的任务进行后处理
                    if let TaskStatus::Finished = task_status {
                        unfinished_tasks.remove(&task_type);
                    }
                    if unfinished_tasks.is_empty() {
                        tracing::info!(
                            "All tasks finished, start post processing for file: {}",
                            file_identifier,
                        );
                        let _ = task_post_process(
                            &ctx,
                            &file_identifier,
                            &payload.metadata,
                            surrealdb_client.clone(),
                        )
                        .await;
                    }
                }
            }
            .instrument(tracing::Span::current())
        });

        Ok(notification_rx)
    }
}

#[tracing::instrument(level = "info", skip_all)]
async fn task_post_process(
    ctx: &ContentBaseCtx,
    file_identifier: &str,
    metadata: &ContentMetadata,
    surrealdb_client: Arc<RwLock<DB>>,
) -> anyhow::Result<()> {
    match metadata {
        ContentMetadata::Video(metadata) => {
            // 但如果 video 没有音频，则直接跳过 TransChunkSumEmbed
            upsert_video_index_to_surrealdb(ctx, file_identifier, metadata, surrealdb_client)
                .await?;
            tracing::info!("video index upserted to surrealdb");
        }
        ContentMetadata::Audio(metadata) => {
            upsert_audio_index_to_surrealdb(ctx, file_identifier, metadata, surrealdb_client)
                .await?;
            tracing::info!("audio index upserted to surrealdb");
        }
        ContentMetadata::Image(metadata) => {
            // DescEmbed 和 Embedding 结束后都触发 upsert_image_index_to_surrealdb
            // 如果有一个任务没完成，upsert_image_index_to_surrealdb 会报错
            upsert_image_index_to_surrealdb(ctx, file_identifier, metadata, surrealdb_client)
                .await?;
            tracing::info!("image index upserted to surrealdb");
        }
        ContentMetadata::RawText(metadata) => {
            upsert_document_index_to_surrealdb(ctx, file_identifier, metadata, surrealdb_client)
                .await?;
            tracing::info!("document index upserted to surrealdb");
        }
        ContentMetadata::WebPage(metadata) => {
            upsert_web_page_index_to_surrealdb(ctx, file_identifier, metadata, surrealdb_client)
                .await?;
            tracing::info!("web page index upserted to surrealdb");
        }
        _ => {}
    };
    Ok(())
}

fn warn_and_skip(msg: &'static str) -> impl FnOnce(anyhow::Error) -> anyhow::Error {
    move |e: anyhow::Error| {
        tracing::warn!(error = ?e, "Failed to read {} output, skip ... ", msg); // error = %e
        e
    }
}

#[tracing::instrument(skip_all)]
async fn upsert_audio_index_to_surrealdb(
    ctx: &ContentBaseCtx,
    file_identifier: &str,
    _metadata: &AudioMetadata,
    surrealdb_client: Arc<RwLock<DB>>,
) -> anyhow::Result<()> {
    let chunks = AudioTransChunkTask
        .chunk_content(file_identifier, ctx)
        .await
        .map_err(warn_and_skip("audio transcript chunks"))?;
    let future = chunks
        .into_iter()
        .map(|chunk| {
            async move {
                let start_timestamp = chunk.start_timestamp;
                let end_timestamp = chunk.end_timestamp;
                // let content = chunk.text;
                // 不直接使用 transcript 文本，使用 transcript summary 文本，和 embedding 对应
                // 另一个原因是现在中文的全文搜索不大好，所以都用英文总结的 transcript summary
                let content = AudioTransChunkSumTask
                    .sum_content(file_identifier, ctx, start_timestamp, end_timestamp)
                    .await
                    .map_err(warn_and_skip("audio transcript chunk summary"))?;
                let embedding = AudioTransChunkSumEmbedTask
                    .embed_content(file_identifier, ctx, start_timestamp, end_timestamp)
                    .await
                    .map_err(warn_and_skip("audio transcript chunk summary embedding"))?;
                let texts = vec![TextModel {
                    id: None,
                    content,
                    embedding,
                    // en_content: "".to_string(),
                    // en_embedding: vec![],
                }];
                let audio_frame = AudioFrameModel {
                    id: None,
                    start_timestamp,
                    end_timestamp,
                };
                anyhow::Result::<(AudioFrameModel, Vec<TextModel>)>::Ok((audio_frame, texts))
            }
            .instrument(tracing::Span::current())
        })
        .collect::<Vec<_>>();
    let audio_frames: anyhow::Result<Vec<(AudioFrameModel, Vec<TextModel>)>> =
        collect_async_results!(future);
    surrealdb_client
        .try_write()?
        .insert_audio(
            file_identifier.to_string(),
            (AudioModel { id: None }, audio_frames?),
        )
        .await?;
    Ok(())
}

#[tracing::instrument(skip_all)]
async fn upsert_video_index_to_surrealdb(
    ctx: &ContentBaseCtx,
    file_identifier: &str,
    metadata: &VideoMetadata,
    surrealdb_client: Arc<RwLock<DB>>,
) -> anyhow::Result<()> {
    let audio_frames: Vec<(AudioFrameModel, Vec<TextModel>)> = if metadata.audio.is_some() {
        let chunks = VideoTransChunkTask
            .chunk_content(file_identifier, ctx)
            .await
            .map_err(warn_and_skip("video transcript chunks"))?;
        // tracing::debug!("video chunks: {chunks:?}");
        let future = chunks
            .into_iter()
            .map(|chunk| {
                async move {
                    let start_timestamp = chunk.start_timestamp;
                    let end_timestamp = chunk.end_timestamp;
                    // let content = chunk.text;
                    // 不直接使用 transcript 文本，使用 transcript summary 文本，和 embedding 对应
                    // 另一个原因是现在中文的全文搜索不大好，所以都用英文总结的 transcript summary
                    let content = VideoTransChunkSumTask
                        .sum_content(file_identifier, ctx, start_timestamp, end_timestamp)
                        .await
                        .map_err(warn_and_skip("video transcript chunk summary"))?;
                    let embedding = VideoTransChunkSumEmbedTask
                        .embed_content(file_identifier, ctx, start_timestamp, end_timestamp)
                        .await
                        .map_err(warn_and_skip("video transcript summary embedding"))?;
                    let audio_frame = AudioFrameModel {
                        id: None,
                        start_timestamp,
                        end_timestamp,
                    };
                    let texts = vec![TextModel {
                        id: None,
                        content,
                        embedding,
                        // en_content: "".to_string(),
                        // en_embedding: vec![],
                    }];
                    Result::<(AudioFrameModel, Vec<TextModel>), anyhow::Error>::Ok((
                        audio_frame,
                        texts,
                    ))
                }
                .instrument(tracing::Span::current())
            })
            .collect::<Vec<_>>();
        try_join_all(future).await?
    } else {
        vec![]
    };

    let image_frames: Vec<(ImageFrameModel, Vec<ImageModel>)> = {
        let frames = VideoFrameTask
            .frame_content(file_identifier, ctx)
            .await
            .map_err(warn_and_skip("video image frames"))?;
        // tracing::debug!("video frames: {frames:?}");
        let future = frames
            .chunks(VIDEO_FRAME_SUMMARY_BATCH_SIZE)
            .into_iter()
            .map(|frame_infos_chunk| {
                async move {
                    let first_frame = frame_infos_chunk.first().expect("first chunk should exist");
                    let last_frame = frame_infos_chunk.last().expect("last chunk should exist");
                    let start_timestamp = first_frame.timestamp;
                    let end_timestamp = last_frame.timestamp;
                    let caption_embedding = VideoFrameDescEmbedTask
                        .frame_desc_embed_content(
                            file_identifier,
                            ctx,
                            start_timestamp,
                            end_timestamp,
                        )
                        .await
                        .map_err(warn_and_skip("video frame desc embedding"))?;
                    let caption = VideoFrameDescriptionTask
                        .frame_description_content(
                            file_identifier,
                            ctx,
                            start_timestamp,
                            end_timestamp,
                        )
                        .await
                        .map_err(warn_and_skip("video frame description"))?;
                    // TODO: 优化?, 目前 embedding 只取第一个 chunk 的，description 取的是一个片段的
                    let embedding = VideoFrameEmbeddingTask
                        .frame_embedding_content(file_identifier, ctx, start_timestamp)
                        .await
                        .map_err(warn_and_skip("video frame embedding"))?;
                    let image_frame = ImageFrameModel {
                        id: None,
                        start_timestamp,
                        end_timestamp,
                    };
                    let images = vec![ImageModel {
                        id: None,
                        caption,
                        embedding,
                        caption_embedding,
                    }];
                    Result::<(ImageFrameModel, Vec<ImageModel>), anyhow::Error>::Ok((
                        image_frame,
                        images,
                    ))
                }
                .instrument(tracing::Span::current())
            });
        try_join_all(future).await?
    };

    surrealdb_client
        .try_write()?
        .insert_video(
            file_identifier.to_string(),
            (VideoModel { id: None }, image_frames, audio_frames),
        )
        .await?;
    Ok(())
}

#[tracing::instrument(skip_all)]
async fn upsert_image_index_to_surrealdb(
    ctx: &ContentBaseCtx,
    file_identifier: &str,
    _metadata: &ImageMetadata,
    surrealdb_client: Arc<RwLock<DB>>,
) -> anyhow::Result<()> {
    // 不用 task_type.desc_embed_content，用 ImageDescEmbedTask 创建个空实例，统一写法
    let caption_embedding = ImageDescEmbedTask
        .desc_embed_content(file_identifier, ctx)
        .await
        .map_err(warn_and_skip("image desc embedding"))?;
    let caption = ImageDescriptionTask
        .description_content(file_identifier, ctx)
        .await
        .map_err(warn_and_skip("image description"))?;
    let embedding = ImageEmbeddingTask
        .embedding_content(file_identifier, ctx)
        .await
        .map_err(warn_and_skip("image embedding"))?;
    surrealdb_client
        .try_write()?
        .insert_image(
            file_identifier.to_string(),
            ImageModel {
                id: None,
                caption,
                embedding,
                caption_embedding,
            },
        )
        .await?;
    Ok(())
}

#[tracing::instrument(skip_all)]
async fn upsert_document_index_to_surrealdb(
    ctx: &ContentBaseCtx,
    file_identifier: &str,
    _metadata: &RawTextMetadata,
    surrealdb_client: Arc<RwLock<DB>>,
) -> anyhow::Result<()> {
    let chunks = RawTextChunkTask
        .chunk_content(file_identifier, ctx)
        .await
        .map_err(warn_and_skip("document chunk"))?;
    let futures = chunks
        .into_iter()
        .enumerate()
        .map(|(i, content)| {
            async move {
                let embedding = RawTextChunkSumEmbedTask
                    .embed_content(file_identifier, ctx, i)
                    .await
                    .map_err(warn_and_skip("document chunk summary embedding"))?;
                let texts = vec![TextModel {
                    id: None,
                    content,
                    embedding,
                    // en_content: "".to_string(),
                    // en_embedding: vec![],
                }];
                let images: Vec<ImageModel> = vec![];
                let page = PageModel {
                    id: None,
                    start_index: i,
                    end_index: i,
                };
                anyhow::Result::<(PageModel, Vec<TextModel>, Vec<ImageModel>)>::Ok((
                    page, texts, images,
                ))
            }
            .instrument(tracing::Span::current())
        })
        .collect::<Vec<_>>();
    let pages: anyhow::Result<Vec<(PageModel, Vec<TextModel>, Vec<ImageModel>)>> =
        collect_async_results!(futures);
    surrealdb_client
        .try_write()?
        .insert_document(
            file_identifier.to_string(),
            (DocumentModel { id: None }, pages?),
        )
        .await?;
    Ok(())
}

#[tracing::instrument(skip_all)]
async fn upsert_web_page_index_to_surrealdb(
    ctx: &ContentBaseCtx,
    file_identifier: &str,
    _metadata: &WebPageMetadata,
    surrealdb_client: Arc<RwLock<DB>>,
) -> anyhow::Result<()> {
    let chunks = WebPageChunkTask.chunk_content(file_identifier, ctx).await?;
    let futures = chunks
        .into_iter()
        .enumerate()
        .map(|(i, content)| {
            async move {
                let embedding = WebPageChunkSumEmbedTask
                    .embed_content(file_identifier, ctx, i)
                    .await
                    .map_err(warn_and_skip("web page chunk summary embedding"))?;
                let texts = vec![TextModel {
                    id: None,
                    content,
                    embedding,
                    // en_content: "".to_string(),
                    // en_embedding: vec![],
                }];
                let images: Vec<ImageModel> = vec![];
                let page = PageModel {
                    id: None,
                    start_index: i,
                    end_index: i,
                };
                anyhow::Result::<(PageModel, Vec<TextModel>, Vec<ImageModel>)>::Ok((
                    page, texts, images,
                ))
            }
            .instrument(tracing::Span::current())
        })
        .collect::<Vec<_>>();
    let pages: anyhow::Result<Vec<(PageModel, Vec<TextModel>, Vec<ImageModel>)>> =
        collect_async_results!(futures);
    surrealdb_client
        .try_write()?
        .insert_web_page(
            file_identifier.to_string(),
            (WebPageModel { id: None }, pages?),
        )
        .await?;
    Ok(())
}
