use std::sync::Arc;
use tokio::sync::broadcast::{self, Sender};
use rspc::Router;
use tracing::{debug, info, error};
use crate::{Ctx, R};
use prisma_lib::{
    PrismaClient,
    new_client,
    video_task,
    VideoTaskType,
};
use file_handler::video::VideoHandler;

#[derive(prisma_client_rust::specta::Type)]
#[specta(crate="prisma_client_rust::specta")]
pub struct VideoTaskResult {
    pub id: i32,
    pub video_path: String,
}

pub fn get_routes() -> Router<Ctx> {
    let tx = init_task_pool();
    R.router()
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
        .procedure(
            "list_video_tasks",
            R.query(move |_ctx: Ctx, _input: ()| async move {
                let client = new_client().await.expect("failed to create prisma client");
                let res = client.video_task().find_many(vec![]).exec().await.expect("failed to list video tasks");
                // return VideoTaskResult {
                //     id: 1,
                //     video_path: "abc".to_owned()
                // };
                // TODO: 这里没搞定返回结果类型的定义，可能是因为现在用的 rspc 的版本和 prisma rust client 版本不兼容
                serde_json::to_value(res).unwrap()
            })
        )
}

#[derive(Clone)]
pub struct TaskPayload {
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


async fn save_starts_at(task_type: VideoTaskType, client: &PrismaClient, vh: &VideoHandler) {
    client.video_task().update(
        video_task::video_file_hash_task_type(
            String::from(vh.file_identifier()),
            task_type
        ),
        vec![video_task::starts_at::set(Some(chrono::Utc::now().into()))]
    ).exec().await.expect(&format!("failed save_starts_at {:?}", task_type));
}

async fn save_ends_at(task_type: VideoTaskType, client: &PrismaClient, vh: &VideoHandler) {
    client.video_task().update(
        video_task::video_file_hash_task_type(
            String::from(vh.file_identifier()),
            task_type
        ),
        vec![video_task::ends_at::set(Some(chrono::Utc::now().into()))]
    ).exec().await.expect(&format!("failed save_ends_at {:?}", task_type));
}

async fn process_task(task_payload: &TaskPayload) {
    // sleep for random time
    // let sleep_time = rand::random::<u64>() % 10;
    // tokio::time::sleep(tokio::time::Duration::from_secs(sleep_time)).await;
    // info!("Task finished {}", &task_payload.video_path);
    let client: PrismaClient = new_client().await.expect("failed to create prisma client");
    let client = Arc::new(client);
    let vh: &VideoHandler = &task_payload.video_handler;

    save_starts_at(VideoTaskType::Frame, &client, vh).await;
    if let Err(e) = vh.get_frames().await {
        error!("failed to get frames: {}", e);
        return;
    }
    info!("successfully got frames, {}", &task_payload.video_path);
    save_ends_at(VideoTaskType::Frame, &client, vh).await;

    save_starts_at(VideoTaskType::FrameCaption, &client, vh).await;
    if let Err(e) = vh.get_frames_caption().await {
        error!("failed to get frames caption: {}", e);
        return;
    }
    info!("successfully got frames caption, {}", &task_payload.video_path);
    save_ends_at(VideoTaskType::FrameCaption, &client, vh).await;

    save_starts_at(VideoTaskType::Audio, &client, vh).await;
    if let Err(e) = vh.get_audio().await {
        error!("failed to get audio: {}", e);
        return;
    }
    info!("successfully got audio, {}", &task_payload.video_path);
    save_ends_at(VideoTaskType::Audio, &client, vh).await;

    save_starts_at(VideoTaskType::Transcript, &client, vh).await;
    if let Err(e) = vh.get_transcript().await {
        error!("failed to get transcript: {}", e);
        return;
    }
    info!("successfully got transcript, {}", &task_payload.video_path);
    save_ends_at(VideoTaskType::Transcript, &client, vh).await;
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

    let client = new_client().await.expect("failed to create prisma client");

    for task_type in vec![
        VideoTaskType::Frame, VideoTaskType::FrameCaption, VideoTaskType::Audio, VideoTaskType::Transcript
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
