use std::sync::Arc;
use crate::CtxWithLibrary;
use file_handler::video::VideoHandler;
use prisma_lib::{asset_object, file_handler_task, PrismaClient};
use tokio::sync::broadcast::{self, Sender};
use tracing::{
    error,
    // debug,
    info,
};

pub enum VideoTaskType {
    Frame,
    FrameCaption,
    FrameContentEmbedding,
    FrameCaptionEmbedding,
    Audio,
    Transcript,
    TranscriptEmbedding,
}

impl ToString for VideoTaskType {
    fn to_string(&self) -> String {
        match self {
            VideoTaskType::Frame => "Frame".to_string(),
            VideoTaskType::FrameCaption => "FrameCaption".to_string(),
            VideoTaskType::FrameContentEmbedding => "FrameContentEmbedding".to_string(),
            VideoTaskType::FrameCaptionEmbedding => "FrameCaptionEmbedding".to_string(),
            VideoTaskType::Audio => "Audio".to_string(),
            VideoTaskType::Transcript => "Transcript".to_string(),
            VideoTaskType::TranscriptEmbedding => "TranscriptEmbedding".to_string(),
        }
    }
}

#[derive(Clone)]
pub struct TaskPayload {
    pub prisma_client: Arc<PrismaClient>,
    pub asset_object_id: i32,
    pub video_handler: VideoHandler,
    pub file_path: String,
}

pub fn init_task_pool() -> Arc<broadcast::Sender<TaskPayload>> {
    let (tx, _rx) = broadcast::channel::<TaskPayload>(500);
    let tx = Arc::new(tx);
    let mut rx = tx.subscribe();
    tokio::spawn(async move {
        loop {
            match rx.recv().await {
                Ok(task_payload) => {
                    tracing::info!("Task received: {:?}", task_payload.file_path);
                    process_task(&task_payload).await;
                }
                Err(e) => {
                    tracing::error!("No Task Error: {:?}", e);
                }
            }
        }
    });
    tx
}

async fn save_starts_at(asset_object_id: i32, task_type: &str, client: Arc<PrismaClient>) {
    client
        .file_handler_task()
        .update(
            file_handler_task::asset_object_id_task_type(
                asset_object_id,
                task_type.to_string(),
            ),
            vec![file_handler_task::starts_at::set(Some(chrono::Utc::now().into()))],
        )
        .exec()
        .await
        .expect(&format!("failed save_starts_at {:?}", task_type));
}

async fn save_ends_at(asset_object_id: i32, task_type: &str, client: Arc<PrismaClient>) {
    client
        .file_handler_task()
        .update(
            file_handler_task::asset_object_id_task_type(
                asset_object_id,
                task_type.to_string(),
            ),
            vec![file_handler_task::ends_at::set(Some(chrono::Utc::now().into()))],
        )
        .exec()
        .await
        .expect(&format!("failed save_ends_at {:?}", task_type));
}

async fn process_task(task_payload: &TaskPayload) {
    // sleep for random time
    // let sleep_time = rand::random::<u64>() % 10;
    // tokio::time::sleep(tokio::time::Duration::from_secs(sleep_time)).await;
    // info!("Task finished {}", &task_payload.video_path);
    let vh: &VideoHandler = &task_payload.video_handler;

    for task_type in [
        VideoTaskType::Frame,
        VideoTaskType::FrameContentEmbedding,
        VideoTaskType::FrameCaption,
        VideoTaskType::FrameCaptionEmbedding,
        VideoTaskType::Audio,
        VideoTaskType::Transcript,
        VideoTaskType::TranscriptEmbedding,
    ] {
        save_starts_at(
            task_payload.asset_object_id,
            &task_type.to_string(),
            Arc::clone(&task_payload.prisma_client)
        ).await;
        let result = match task_type {
            VideoTaskType::Frame => vh.get_frames().await,
            VideoTaskType::FrameContentEmbedding => vh.get_frame_content_embedding().await,
            VideoTaskType::FrameCaption => vh.get_frames_caption().await,
            VideoTaskType::FrameCaptionEmbedding => vh.get_frame_caption_embedding().await,
            VideoTaskType::Audio => vh.get_audio().await,
            VideoTaskType::Transcript => vh.get_transcript().await,
            VideoTaskType::TranscriptEmbedding => vh.get_transcript_embedding().await,
        };
        if let Err(e) = result {
            error!("Task failed: {}, {}, {}", &task_type.to_string(), &task_payload.file_path, e);
        } else {
            info!("Task success: {}, {}", &task_type.to_string(), &task_payload.file_path);
        }
        save_ends_at(
            task_payload.asset_object_id,
            &task_type.to_string(),
            Arc::clone(&task_payload.prisma_client)
        ).await;
    }
}

pub async fn create_video_task<TCtx>(
    materialized_path: &str,
    asset_object_data: &asset_object::Data,
    ctx: &TCtx,
    tx: Arc<Sender<TaskPayload>>,
) -> Result<(), ()>
where
    TCtx: CtxWithLibrary + Clone + Send + Sync + 'static,
{
    let library = &ctx.library().map_err(|e| {
        error!("library must be set before triggering create_video_task: {}", e);
    })?;

    let local_video_file_full_path = format!(
        "{}/{}",
        library.files_dir.to_str().unwrap(),
        asset_object_data.id
    );

    let video_handler = match VideoHandler::new(
        local_video_file_full_path,
        &ctx.get_resources_dir(),
        &library,
    )
    .await
    {
        Ok(vh) => vh,
        Err(e) => {
            error!("failed to initialize video handler: {}", e);
            return Err(());
        }
    };

    let hash = video_handler.file_identifier().to_string();
    if let Err(e) = library.prisma_client()
        .asset_object()
        .update(
            asset_object::id::equals(asset_object_data.id),
            vec![asset_object::hash::set(Some(hash))],
        )
        .exec()
        .await
    {
        error!("failed to update asset_object hash: {}", e);
        return Err(());
    }

    for task_type in vec![
        VideoTaskType::Frame,
        VideoTaskType::FrameContentEmbedding,
        VideoTaskType::FrameCaptionEmbedding,
        VideoTaskType::FrameCaption,
        VideoTaskType::Audio,
        VideoTaskType::Transcript,
        VideoTaskType::TranscriptEmbedding,
    ] {
        let x = library.prisma_client()
        .file_handler_task().upsert(
            file_handler_task::asset_object_id_task_type(
                asset_object_data.id,
                task_type.to_string(),
            ),
            file_handler_task::create(
                asset_object_data.id,
                task_type.to_string(),
                vec![],
            ),
            vec![
                file_handler_task::starts_at::set(None),
                file_handler_task::ends_at::set(None),
            ],
        ).exec().await;

        match x {
            Ok(res) => {
                info!("Task created: {:?}", res);
            }
            Err(e) => {
                error!("Failed to create task: {}", e);
            }
        }
    }

    let task_payload = TaskPayload {
        file_path: materialized_path.to_string(),
        asset_object_id: asset_object_data.id,
        prisma_client: library.prisma_client(),
        video_handler,
    };

    match tx.send(task_payload) {
        Ok(rem) => {
            info!("Task queued {}, remaining receivers {}", materialized_path, rem);
        }
        Err(e) => {
            error!("Failed to queue task {}: {}", materialized_path, e);
        }
    };

    Ok(())
}
