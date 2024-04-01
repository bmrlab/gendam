mod pool;
mod priority;

use crate::{
    task_queue::priority::{TaskPriority, TaskPriorityRaw},
    CtxWithLibrary,
};
use file_handler::video::{VideoHandler, VideoTaskType};
pub use pool::*;
use prisma_lib::{asset_object, file_handler_task};
use tracing::{error, info};

pub async fn create_video_task(
    asset_object_data: &asset_object::Data,
    ctx: &impl CtxWithLibrary,
) -> Result<(), ()> {
    let library = &ctx.library().map_err(|e| {
        error!(
            "library must be set before triggering create_video_task: {}",
            e
        );
    })?;

    let local_video_file_full_path = library.file_path(&asset_object_data.hash);

    let ai_handler = ctx.get_ai_handler();

    let video_handler = match VideoHandler::new(
        local_video_file_full_path,
        &asset_object_data.hash,
        &library,
    ) {
        Ok(vh) => vh
            .with_clip(ai_handler.clip)
            .with_blip(ai_handler.blip)
            .with_whisper(ai_handler.whisper),
        Err(e) => {
            error!("failed to initialize video handler: {}", e);
            return Err(());
        }
    };

    tracing::debug!("asset object: {:?}", asset_object_data);

    let video_has_audio = match asset_object_data.media_data() {
        Ok(Some(metadata)) => metadata.has_audio,
        _ => None,
    };

    for task_type in video_handler.get_supported_task_types(video_has_audio) {
        let x = library
            .prisma_client()
            .file_handler_task()
            .upsert(
                file_handler_task::asset_object_id_task_type(
                    asset_object_data.id,
                    task_type.to_string(),
                ),
                file_handler_task::create(asset_object_data.id, task_type.to_string(), vec![]),
                vec![
                    file_handler_task::starts_at::set(None),
                    file_handler_task::ends_at::set(None),
                    file_handler_task::exit_code::set(None),
                    file_handler_task::exit_message::set(None),
                ],
            )
            .exec()
            .await;

        match x {
            Ok(res) => {
                info!("Task created: {:?}", res);
            }
            Err(e) => {
                error!("Failed to create task: {}", e);
            }
        }
    }

    let tx = ctx.get_task_tx();

    match tx.lock() {
        Ok(tx) => {
            for task_type in video_handler.get_supported_task_types(video_has_audio) {
                let priority = match task_type {
                    VideoTaskType::FrameCaption
                    | VideoTaskType::FrameCaptionEmbedding
                    | VideoTaskType::TranscriptEmbedding => TaskPriority::new(TaskPriorityRaw::Low),
                    VideoTaskType::Frame
                    | VideoTaskType::FrameContentEmbedding
                    | VideoTaskType::Audio
                    | VideoTaskType::Transcript => TaskPriority::new(TaskPriorityRaw::Normal),
                };

                match tx.send(TaskPayload::Task((
                    Task {
                        handler: video_handler.clone(),
                        task_type: task_type.clone(),
                        asset_object_id: asset_object_data.id,
                        prisma_client: library.prisma_client(),
                    },
                    priority,
                ))) {
                    Ok(_) => {
                        info!(
                            "Task queued {} {}, priority: {}",
                            asset_object_data.id, &task_type, priority
                        );
                    }
                    Err(e) => {
                        error!(
                            "Failed to queue task {} {}: {}",
                            asset_object_data.id, &task_type, e
                        );
                    }
                }
            }
        }
        Err(e) => {
            error!("Failed to lock mutex: {}", e);
        }
    }

    Ok(())
}
