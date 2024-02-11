use std::sync::Arc;
use tokio::sync::broadcast::{self, Sender};
use rspc::Router;
use tracing::{debug, info, error};
use crate::{Ctx, R};
use prisma_lib::{
    new_client,
    video_task,
    VideoTaskType,
};

pub fn get_routes() -> Router<Ctx> {
    let tx = init_task_pool();
    R.router()
        .procedure(
            "create_video_frames",
            R.mutation(|ctx, video_path: String| async move {
                let res = create_video_frames(&ctx, &video_path).await;
                serde_json::to_value(res).unwrap()
            })
        )
        .procedure(
            "create_video_task",
            R.mutation(move |ctx: Ctx, video_path: String| {
                let tx2 = Arc::clone(&tx);
                async move {
                    let res = create_video_task(&ctx, &video_path, tx2).await;
                    serde_json::to_value(res).unwrap()
                }
            })
        )
}

async fn create_video_frames(ctx: &Ctx, video_path: &str) {
    let video_handler =
        file_handler::video::VideoHandler::new(
            video_path,
            &ctx.local_data_dir,
            &ctx.resources_dir,
        )
        .await
        .expect("failed to initialize video handler");
    let frame_handle = tokio::spawn(async move {
        match video_handler.get_frames().await {
            Ok(res) => {
                debug!("successfully got frames");
                Ok(res)
            },
            Err(e) => {
                debug!("failed to get frames: {}", e);
                Err(e)
            }
        }
    });
    let result = frame_handle.await.unwrap();
    result.expect("failed to get frames");
}

#[derive(Clone)]
pub struct TaskPayload {
    pub video_handler: file_handler::video::VideoHandler,
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

async fn process_task(task_payload: &TaskPayload) {
    // sleep for random time
    // let sleep_time = rand::random::<u64>() % 10;
    // tokio::time::sleep(tokio::time::Duration::from_secs(sleep_time)).await;
    // info!("Task finished {}", &task_payload.video_path);
    let client = new_client().await.expect("failed to create prisma client");
    let vh = &task_payload.video_handler;

    client.video_task().update(
        video_task::video_file_hash_task_type(
            String::from(vh.file_identifier()),
            VideoTaskType::Frames
        ),
        vec![video_task::starts_at::set(Some(chrono::Utc::now().into()))]
    ).exec().await.expect("failed to update video task");

    match vh.get_frames().await {
        Ok(_res) => {
            debug!("successfully got frames, {}", &task_payload.video_path);
            client.video_task().update(
                video_task::video_file_hash_task_type(
                    String::from(vh.file_identifier()),
                    VideoTaskType::Frames
                ),
                vec![video_task::ends_at::set(Some(chrono::Utc::now().into()))]
            ).exec().await.expect("failed to update video task");
        },
        Err(e) => {
            debug!("failed to get frames: {}", e);
        }
    };

    client.video_task().update(
        video_task::video_file_hash_task_type(
            String::from(vh.file_identifier()),
            VideoTaskType::Audio
        ),
        vec![video_task::starts_at::set(Some(chrono::Utc::now().into()))]
    ).exec().await.expect("failed to update video task");
    match vh.get_audio().await {
        // `get_transcript` 使用whisper提取音频
        Ok(_) => {
            debug!("successfully got audio, {}", &task_payload.video_path);
            client.video_task().update(
                video_task::video_file_hash_task_type(
                    String::from(vh.file_identifier()),
                    VideoTaskType::Audio
                ),
                vec![video_task::ends_at::set(Some(chrono::Utc::now().into()))]
            ).exec().await.expect("failed to update video task");
            client.video_task().update(
                video_task::video_file_hash_task_type(
                    String::from(vh.file_identifier()),
                    VideoTaskType::Transcript
                ),
                vec![video_task::starts_at::set(Some(chrono::Utc::now().into()))]
            ).exec().await.expect("failed to update video task");
            match vh.get_transcript().await {
                Ok(_) => {
                    debug!("successfully got transcript, {}", &task_payload.video_path);
                    client.video_task().update(
                        video_task::video_file_hash_task_type(
                            String::from(vh.file_identifier()),
                            VideoTaskType::Transcript
                        ),
                        vec![video_task::ends_at::set(Some(chrono::Utc::now().into()))]
                    ).exec().await.expect("failed to update video task");
                }
                Err(e) => {
                    error!("failed to get transcript: {}", e);
                }
            }
        },
        Err(e) => {
            error!("failed to get audio: {}", e);
        }
    };
}

async fn create_video_task(
    ctx: &Ctx,
    video_path: &str,
    tx: Arc<Sender<TaskPayload>>
) {
    let video_handler =
        file_handler::video::VideoHandler::new(
            video_path,
            &ctx.local_data_dir,
            &ctx.resources_dir,
        )
        .await
        .expect("failed to initialize video handler");

    let client = new_client().await.expect("failed to create prisma client");

    for task_type in vec![
        VideoTaskType::Frames, VideoTaskType::Audio, VideoTaskType::Transcript
    ] {
        let x = client.video_task().upsert(
            video_task::video_file_hash_task_type(
                String::from(video_handler.file_identifier()),
                task_type
            ),
            video_task::create(
                video_path.to_owned(),
                String::from(video_handler.file_identifier()),
                task_type,
                vec![],
            ),
            vec![],
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
