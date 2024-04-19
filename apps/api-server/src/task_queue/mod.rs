mod pool;
mod priority;

use crate::{
    task_queue::priority::{TaskPriority, TaskPriorityRaw},
    CtxWithLibrary,
};
use content_library::Library;
use file_handler::{video::{VideoHandler, VideoTaskType}, FileHandler};
pub use pool::*;
use prisma_lib::{
    asset_object::{self, media_data},
    file_handler_task,
};
use std::{collections::HashMap, str::FromStr};
use tracing::{error, info};

pub trait Handler {
    #[allow(async_fn_in_trait)]
    async fn process(&self, task_type: &str) -> anyhow::Result<()>;
}

impl Handler for VideoHandler {
    async fn process(&self, task_type: &str) -> anyhow::Result<()> {
        self.run_task(&VideoTaskType::from_str(task_type)?).await
    }
}

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
        Some(task_types) => valid_task_types
            .into_iter()
            .filter(|v| task_types.contains(v))
            .collect(),
        None => valid_task_types,
    };

    for task_type in task_types.clone() {
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

    let tx = ctx.task_tx().map_err(|_| ())?;

    for (idx, task_type) in task_types.iter().enumerate() {
        let priority = match task_type {
            // it's better not to add default arm `_ => {}` here
            // if there are new task type in future, compiler will throw an error
            // this will force us to add priority manually
            VideoTaskType::FrameCaption | VideoTaskType::FrameCaptionEmbedding => {
                TaskPriority::new(TaskPriorityRaw::Low)
            }
            VideoTaskType::Frame
            | VideoTaskType::FrameContentEmbedding
            | VideoTaskType::Audio
            | VideoTaskType::Transcript
            | VideoTaskType::TranscriptEmbedding
            | VideoTaskType::FrameTags
            | VideoTaskType::FrameTagsEmbedding => TaskPriority::new(TaskPriorityRaw::Normal),
        };

        match tx.send(TaskPayload::Task((
            Task {
                handler: video_handler.clone(),
                task_type: task_type.clone(),
                asset_object_id: asset_object_data.id,
                prisma_client: library.prisma_client(),
            },
            priority,
            idx,
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
