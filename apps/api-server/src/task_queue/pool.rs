use std::sync::Arc;
use crate::CtxWithLibrary;
use file_handler::video::VideoHandler;
use prisma_lib::{video_task, PrismaClient};
use tokio::sync::{
    broadcast::{self, Sender},
    RwLock,
};
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
    pub prisma_client: Arc<RwLock<PrismaClient>>,
    pub video_handler: VideoHandler,
    pub video_path: String,
    // pub video_file_hash: String,
    // pub task_type: VideoTaskType,
}

pub fn init_task_pool() -> Arc<broadcast::Sender<TaskPayload>> {
    let (tx, _rx) = broadcast::channel::<TaskPayload>(500);
    let tx = Arc::new(tx);
    let mut rx = tx.subscribe();
    tokio::spawn(async move {
        loop {
            match rx.recv().await {
                Ok(task_payload) => {
                    tracing::info!("Task received: {:?}", task_payload.video_path);
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

async fn save_starts_at(task_type: &str, prisma_client: Arc<RwLock<PrismaClient>>, vh: &VideoHandler) {
    let client = prisma_client.write().await;
    client
        .video_task()
        .update(
            video_task::video_file_hash_task_type(
                String::from(vh.file_identifier()),
                task_type.to_string(),
            ),
            vec![video_task::starts_at::set(Some(chrono::Utc::now().into()))],
        )
        .exec()
        .await
        .expect(&format!("failed save_starts_at {:?}", task_type));
}

async fn save_ends_at(task_type: &str, prisma_client: Arc<RwLock<PrismaClient>>, vh: &VideoHandler) {
    let client = prisma_client.write().await;
    client
        .video_task()
        .update(
            video_task::video_file_hash_task_type(
                String::from(vh.file_identifier()),
                task_type.to_string(),
            ),
            vec![video_task::ends_at::set(Some(chrono::Utc::now().into()))],
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

    save_starts_at(&VideoTaskType::Frame.to_string(), Arc::clone(&task_payload.prisma_client), vh).await;
    if let Err(e) = vh.get_frames().await {
        error!("failed to get frames: {}", e);
        // return;
    }
    info!("successfully got frames, {}", &task_payload.video_path);
    save_ends_at(&VideoTaskType::Frame.to_string(), Arc::clone(&task_payload.prisma_client), vh).await;

    save_starts_at(
        &VideoTaskType::FrameContentEmbedding.to_string(),
        Arc::clone(&task_payload.prisma_client),
        vh,
    )
    .await;
    if let Err(e) = vh.get_frame_content_embedding().await {
        error!("failed to get frame content embedding: {}", e);
        // return;
    }
    info!(
        "successfully got frame content embedding, {}",
        &task_payload.video_path
    );
    save_ends_at(
        &VideoTaskType::FrameContentEmbedding.to_string(),
        Arc::clone(&task_payload.prisma_client),
        vh,
    )
    .await;

    save_starts_at(&VideoTaskType::FrameCaption.to_string(), Arc::clone(&task_payload.prisma_client), vh).await;
    if let Err(e) = vh.get_frames_caption().await {
        error!("failed to get frames caption: {}", e);
        // return;
    }
    info!(
        "successfully got frames caption, {}",
        &task_payload.video_path
    );
    save_ends_at(&VideoTaskType::FrameCaption.to_string(), Arc::clone(&task_payload.prisma_client), vh).await;

    save_starts_at(
        &VideoTaskType::FrameCaptionEmbedding.to_string(),
        Arc::clone(&task_payload.prisma_client),
        vh,
    )
    .await;
    if let Err(e) = vh.get_frame_caption_embedding().await {
        error!("failed to get frames caption embedding: {}", e);
        // return;
    }
    info!(
        "successfully got frames caption embedding, {}",
        &task_payload.video_path
    );
    save_ends_at(
        &VideoTaskType::FrameCaptionEmbedding.to_string(),
        Arc::clone(&task_payload.prisma_client),
        vh,
    )
    .await;

    save_starts_at(&VideoTaskType::Audio.to_string(), Arc::clone(&task_payload.prisma_client), vh).await;
    if let Err(e) = vh.get_audio().await {
        error!("failed to get audio: {}", e);
        // return;
    }
    info!("successfully got audio, {}", &task_payload.video_path);
    save_ends_at(&VideoTaskType::Audio.to_string(), Arc::clone(&task_payload.prisma_client), vh).await;

    save_starts_at(&VideoTaskType::Transcript.to_string(), Arc::clone(&task_payload.prisma_client), vh).await;
    if let Err(e) = vh.get_transcript().await {
        error!("failed to get transcript: {}", e);
        // return;
    }
    info!("successfully got transcript, {}", &task_payload.video_path);
    save_ends_at(&VideoTaskType::Transcript.to_string(), Arc::clone(&task_payload.prisma_client), vh).await;

    save_starts_at(&VideoTaskType::TranscriptEmbedding.to_string(), Arc::clone(&task_payload.prisma_client), vh).await;
    if let Err(e) = vh.get_transcript_embedding().await {
        error!("failed to get transcript embedding: {}", e);
        // return;
    }
    info!(
        "successfully got transcript embedding, {}",
        &task_payload.video_path
    );
    save_ends_at(&VideoTaskType::TranscriptEmbedding.to_string(), Arc::clone(&task_payload.prisma_client), vh).await;
}

pub async fn create_video_task<TCtx>(
    ctx: &TCtx,
    video_path: &str,
    tx: Arc<Sender<TaskPayload>>,
) -> Result<(), ()>
where
    TCtx: CtxWithLibrary + Clone + Send + Sync + 'static,
{
    let library = &ctx.library().map_err(|e| {
        error!("library must be set before triggering create_video_task: {}", e);
    })?;

    let video_handler = match VideoHandler::new(
        video_path,
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

    for task_type in vec![
        VideoTaskType::Frame,
        VideoTaskType::FrameContentEmbedding,
        VideoTaskType::FrameCaptionEmbedding,
        VideoTaskType::FrameCaption,
        VideoTaskType::Audio,
        VideoTaskType::Transcript,
        VideoTaskType::TranscriptEmbedding,
    ] {
        let client_w = library.prisma_client.write().await;
        let x = client_w.video_task().upsert(
            video_task::video_file_hash_task_type(
                String::from(video_handler.file_identifier()),
                task_type.to_string(),
            ),
            video_task::create(
                video_path.to_owned(),
                String::from(video_handler.file_identifier()),
                task_type.to_string(),
                vec![],
            ),
            vec![
                video_task::starts_at::set(None),
                video_task::ends_at::set(None),
            ],
        );

        match x.exec().await {
            Ok(res) => {
                info!("Task created: {:?}", res);
            }
            Err(e) => {
                error!("Failed to create task: {}", e);
            }
        }
    }

    let task_payload = TaskPayload {
        prisma_client: Arc::clone(&library.prisma_client),
        video_handler,
        video_path: String::from(video_path),
        // video_file_hash: String::from(video_handler.file_identifier()),
        // task_type: VideoTaskType::Frames,
    };

    match tx.send(task_payload) {
        Ok(rem) => {
            info!("Task queued {}, remaining receivers {}", video_path, rem);
        }
        Err(e) => {
            error!("Failed to queue task {}: {}", video_path, e);
        }
    };

    Ok(())
}
