mod pool;
mod priority;

use crate::{task_queue::priority::TaskPriority, CtxWithLibrary};
use content_library::Library;
use file_handler::{
    video::{VideoHandler, VideoTaskType},
    FileHandler,
};
pub use pool::*;
use prisma_lib::{
    asset_object::{self, media_data},
    file_handler_task,
};
use std::sync::Arc;
use std::{collections::HashMap, str::FromStr};
use tracing::{error, info};

/// 创建视频任务
///
/// `asset_object_data` 需要包含 `media_data` 字段，否则无法正确处理音频
/// `task_types` 不传则会默认采用所有支持的任务类型
pub async fn create_video_task(
    asset_object_data: &asset_object::Data,
    ctx: &impl CtxWithLibrary,
    task_types: Option<Vec<VideoTaskType>>,
) -> Result<(), ()> {
    let qdrant_info = ctx.qdrant_info().map_err(|_| ())?;

    let library = &ctx.library().map_err(|e| {
        error!(
            "library must be set before triggering create_video_task: {}",
            e
        );
    })?;

    let local_video_file_full_path = library.file_path(&asset_object_data.hash);

    let ai_handler = ctx.ai_handler().map_err(|_| ())?;

    let video_handler = match VideoHandler::new(
        local_video_file_full_path,
        &asset_object_data.hash,
        &library,
    ) {
        Ok(vh) => vh
            .with_multi_modal_embedding(
                ai_handler.multi_modal_embedding.as_ref(),
                &qdrant_info.vision_collection.name,
            )
            .with_image_caption(ai_handler.image_caption.as_ref())
            .with_audio_transcript(ai_handler.audio_transcript.as_ref())
            .with_text_embedding(
                ai_handler.text_embedding.as_ref(),
                &qdrant_info.language_collection.name,
            ),
        Err(e) => {
            error!("failed to initialize video handler: {}", e);
            return Err(());
        }
    };

    tracing::debug!("asset object: {:?}", asset_object_data);

    let valid_task_types = video_handler.get_supported_task_types();
    let task_types = match task_types {
        // 这里做一下过滤，防止传入的任务类型不支持或者顺序不正确
        Some(task_types) => {
            let task_types = task_types.iter().map(|v| v.to_string()).collect::<Vec<_>>();
            valid_task_types
                .into_iter()
                .filter(|v| task_types.contains(&v.0))
                .collect()
        }
        None => valid_task_types,
    };

    for task_type in task_types.clone() {
        let x = library
            .prisma_client()
            .file_handler_task()
            .upsert(
                file_handler_task::asset_object_id_task_type(
                    asset_object_data.id,
                    task_type.0.to_string(),
                ),
                file_handler_task::create(asset_object_data.id, task_type.0.to_string(), vec![]),
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

    let tx = ctx.task_tx().map_err(|_| ())?;

    let handler: Arc<Box<dyn FileHandler>> = Arc::new(Box::new(video_handler));

    for (idx, task_type) in task_types.into_iter().enumerate() {
        let priority = TaskPriority::new(task_type.1);

        match tx.send(TaskPayload::Task((
            Task {
                handler: handler.clone(),
                task_type: task_type.0.clone(),
                asset_object_id: asset_object_data.id,
                prisma_client: library.prisma_client(),
            },
            priority,
            idx,
        ))) {
            Ok(_) => {
                info!(
                    "Task queued {} {}, priority: {}",
                    asset_object_data.id, &task_type.0, priority
                );
            }
            Err(e) => {
                error!(
                    "Failed to queue task {} {}: {}",
                    asset_object_data.id, &task_type.0, e
                );
            }
        }
    }

    Ok(())
}

pub async fn trigger_unfinished(
    library: &Library,
    ctx: &impl CtxWithLibrary,
) -> anyhow::Result<()> {
    let tasks = library
        .prisma_client()
        .file_handler_task()
        .find_many(vec![file_handler_task::exit_code::equals(None)])
        // 按照 id 做一下排序，保证任务下次执行的时候还是正确的顺序
        .exec()
        .await?;

    // group tasks by asset_object_id
    let mut asset_tasks: HashMap<i32, Vec<VideoTaskType>> = HashMap::new();
    tasks.into_iter().for_each(|task| {
        if let Ok(task_type) = VideoTaskType::from_str(&task.task_type) {
            if let Some(tasks) = asset_tasks.get_mut(&task.asset_object_id) {
                tasks.push(task_type);
            } else {
                asset_tasks.insert(task.asset_object_id, vec![task_type]);
            }
        }
    });

    // get all asset objects
    let asset_objects = library
        .prisma_client()
        .asset_object()
        .find_many(vec![asset_object::id::in_vec(
            asset_tasks.keys().cloned().collect(),
        )])
        .with(media_data::fetch())
        .exec()
        .await?;

    // trigger tasks
    for asset_object in asset_objects {
        if let Some(tasks) = asset_tasks.get(&asset_object.id).cloned() {
            create_video_task(&asset_object, ctx, Some(tasks))
                .await
                .map_err(|e| anyhow::anyhow!("failed to create video task: {e:?}"))?;
        }
    }

    Ok(())
}
