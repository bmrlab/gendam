use std::sync::Arc;
use tokio::sync::broadcast::{self, Sender};
use rspc::Router;
use tracing::{
    // debug,
    info,
    error
};
use crate::{Ctx, R};
use prisma_lib::{
    PrismaClient,
    new_client_with_url,
    video_task,
};
use prisma_client_rust::Direction;
use file_handler::video::VideoHandler;
use specta::Type;
use serde::Serialize;

pub enum VideoTaskType {
    Frame,
    FrameCaption,
    FrameContentEmbedding,
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
            VideoTaskType::Audio => "Audio".to_string(),
            VideoTaskType::Transcript => "Transcript".to_string(),
            VideoTaskType::TranscriptEmbedding => "TranscriptEmbedding".to_string(),
        }
    }
}

pub fn get_routes() -> Router<Ctx> {
    let tx = init_task_pool();
    R.router()
        .procedure(
            "create",
            R.mutation(move |ctx: Ctx, video_path: String| {
                let tx2 = Arc::clone(&tx);
                async move {
                    let res = create_video_task(&ctx, &video_path, tx2).await;
                    serde_json::to_value(res).unwrap()
                }
            })
        )
        .procedure(
            "list",
            R.query(move |ctx: Ctx, _input: ()| async move {
                let client = new_client_with_url(ctx.db_url.as_str())
                    .await.expect("failed to create prisma client");
                client._db_push().await.expect("failed to push db");  // apply migrations

                let res = client.video_task()
                    .find_many(vec![])
                    .order_by(video_task::id::order(Direction::Desc))
                    .exec().await.expect("failed to list video tasks");

                #[derive(Serialize, Type)]
                pub struct VideoTaskResult {
                    #[serde(rename = "id")]
                    pub id: i32,
                    #[serde(rename = "videoPath")]
                    pub video_path: String,
                    #[serde(rename = "videoFileHash")]
                    pub video_file_hash: String,
                    #[serde(rename = "taskType")]
                    pub task_type: String,
                    #[serde(rename = "startsAt")]
                    pub starts_at: Option<String>,
                    #[serde(rename = "endsAt")]
                    pub ends_at: Option<String>,
                }

                res.iter().map(|item| {
                    VideoTaskResult {
                        id: item.id,
                        video_path: item.video_path.clone(),
                        video_file_hash: item.video_file_hash.clone(),
                        task_type: item.task_type.to_string(),
                        starts_at: if let Some(t) = item.starts_at { Some(t.to_string()) } else { None },
                        ends_at: if let Some(t) = item.ends_at { Some(t.to_string()) } else { None },
                    }
                }).collect::<Vec<_>>()
                // serde_json::to_value(res).unwrap()
            })
        )
}

#[derive(Clone)]
pub struct TaskPayload {
    pub db_url: String,
    pub video_handler: VideoHandler,
    pub video_path: String,
    // pub video_file_hash: String,
    // pub task_type: VideoTaskType,
}

fn init_task_pool() -> Arc<broadcast::Sender<TaskPayload>> {
    let (tx, _rx) = broadcast::channel::<TaskPayload>(500);
    let tx = Arc::new(tx);
    let mut rx = tx.subscribe();
    tokio::spawn(async move {
        loop {
            match rx.recv().await {
                Ok(task_payload) => {
                    tracing::info!("Task received: {:?}", task_payload.video_path);
                    process_task(&task_payload).await;
                },
                Err(e) => {
                    tracing::error!("No Task Error: {:?}", e);
                }
            }
        }
    });
    tx
}

async fn save_starts_at(task_type: &str, client: &PrismaClient, vh: &VideoHandler) {
    client.video_task().update(
        video_task::video_file_hash_task_type(
            String::from(vh.file_identifier()),
            task_type.to_string()
        ),
        vec![video_task::starts_at::set(Some(chrono::Utc::now().into()))]
    ).exec().await.expect(&format!("failed save_starts_at {:?}", task_type));
}

async fn save_ends_at(task_type: &str, client: &PrismaClient, vh: &VideoHandler) {
    client.video_task().update(
        video_task::video_file_hash_task_type(
            String::from(vh.file_identifier()),
            task_type.to_string()
        ),
        vec![video_task::ends_at::set(Some(chrono::Utc::now().into()))]
    ).exec().await.expect(&format!("failed save_ends_at {:?}", task_type));
}

async fn process_task(task_payload: &TaskPayload) {
    // sleep for random time
    // let sleep_time = rand::random::<u64>() % 10;
    // tokio::time::sleep(tokio::time::Duration::from_secs(sleep_time)).await;
    // info!("Task finished {}", &task_payload.video_path);
    let client = new_client_with_url(task_payload.db_url.as_str())
        .await.expect("failed to create prisma client");
    client._db_push().await.expect("failed to push db");  // apply migrations

    let client = Arc::new(client);
    let vh: &VideoHandler = &task_payload.video_handler;

    save_starts_at(&VideoTaskType::Frame.to_string(), &client, vh).await;
    if let Err(e) = vh.get_frames().await {
        error!("failed to get frames: {}", e);
        return;
    }
    info!("successfully got frames, {}", &task_payload.video_path);
    save_ends_at(&VideoTaskType::Frame.to_string(), &client, vh).await;

    save_starts_at(&VideoTaskType::FrameContentEmbedding.to_string(), &client, vh).await;
    if let Err(e) = vh.get_frame_content_embedding().await {
        error!("failed to get frame content embedding: {}", e);
        return;
    }
    info!("successfully got frame content embedding, {}", &task_payload.video_path);
    save_ends_at(&VideoTaskType::FrameContentEmbedding.to_string(), &client, vh).await;

    // save_starts_at(&VideoTaskType::FrameCaption.to_string(), &client, vh).await;
    // if let Err(e) = vh.get_frames_caption().await {
    //     error!("failed to get frames caption: {}", e);
    //     return;
    // }
    // info!("successfully got frames caption, {}", &task_payload.video_path);
    // save_ends_at(&VideoTaskType::FrameCaption.to_string(), &client, vh).await;

    save_starts_at(&VideoTaskType::Audio.to_string(), &client, vh).await;
    if let Err(e) = vh.get_audio().await {
        error!("failed to get audio: {}", e);
        return;
    }
    info!("successfully got audio, {}", &task_payload.video_path);
    save_ends_at(&VideoTaskType::Audio.to_string(), &client, vh).await;

    save_starts_at(&VideoTaskType::Transcript.to_string(), &client, vh).await;
    if let Err(e) = vh.get_transcript().await {
        error!("failed to get transcript: {}", e);
        return;
    }
    info!("successfully got transcript, {}", &task_payload.video_path);
    save_ends_at(&VideoTaskType::Transcript.to_string(), &client, vh).await;

    save_starts_at(&VideoTaskType::TranscriptEmbedding.to_string(), &client, vh).await;
    if let Err(e) = vh.get_transcript_embedding().await {
        error!("failed to get transcript embedding: {}", e);
        return;
    }
    info!("successfully got transcript embedding, {}", &task_payload.video_path);
    save_ends_at(&VideoTaskType::TranscriptEmbedding.to_string(), &client, vh).await;
}

async fn create_video_task(
    ctx: &Ctx,
    video_path: &str,
    tx: Arc<Sender<TaskPayload>>
) {
    let video_handler =
        VideoHandler::new(
            video_path,
            &ctx.local_data_dir,
            &ctx.resources_dir,
        )
        .await
        .expect("failed to initialize video handler");

    let client = new_client_with_url(ctx.db_url.as_str())
        .await.expect("failed to create prisma client");
    client._db_push().await.expect("failed to push db");  // apply migrations

    for task_type in vec![
        VideoTaskType::Frame, VideoTaskType::FrameContentEmbedding,
        VideoTaskType::FrameCaption,
        VideoTaskType::Audio, VideoTaskType::Transcript, VideoTaskType::TranscriptEmbedding,
    ] {
        let x = client.video_task().upsert(
            video_task::video_file_hash_task_type(
                String::from(video_handler.file_identifier()),
                task_type.to_string()
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
            },
            Err(e) => {
                error!("Failed to create task: {}", e);
            }
        }
    }

    let task_payload = TaskPayload {
        db_url: ctx.db_url.clone(),
        video_handler: video_handler,
        video_path: String::from(video_path),
        // video_file_hash: String::from(video_handler.file_identifier()),
        // task_type: VideoTaskType::Frames,
    };

    match tx.send(task_payload) {
        Ok(rem) => {
            info!("Task queued {}, remaining receivers {}", video_path, rem);
        },
        Err(e) => {
            error!("Failed to queue task {}: {}", video_path, e);
        }
    }
}



/*
        .procedure(
            "create_video_frames",
            R.mutation(|ctx, video_path: String| async move {
                let res = create_video_frames(&ctx, &video_path).await;
                serde_json::to_value(res).unwrap()
            })
        )
*/

// async fn create_video_frames(ctx: &Ctx, video_path: &str) {
//     let video_handler =
//         file_handler::video::VideoHandler::new(
//             video_path,
//             &ctx.local_data_dir,
//             &ctx.resources_dir,
//         )
//         .await
//         .expect("failed to initialize video handler");
//     let frame_handle = tokio::spawn(async move {
//         match video_handler.get_frames().await {
//             Ok(res) => {
//                 debug!("successfully got frames");
//                 Ok(res)
//             },
//             Err(e) => {
//                 debug!("failed to get frames: {}", e);
//                 Err(e)
//             }
//         }
//     });
//     let result = frame_handle.await.unwrap();
//     result.expect("failed to get frames");
// }
