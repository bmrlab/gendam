mod pool;
mod priority;

use crate::{
    ai::AIHandler, file_handler::priority::TaskPriority, library::get_library_settings,
    CtxWithLibrary,
};
use anyhow::bail;
use content_library::{Library, QdrantServerInfo};
use file_handler::{video::VideoHandler, FileHandler};
pub use pool::*;
use prisma_client_rust::Direction;
use prisma_lib::{asset_object, file_handler_task};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{error, info};

pub async fn trigger_unfinished(
    library: &Library,
    ctx: &impl CtxWithLibrary,
) -> anyhow::Result<()> {
    let tasks = library
        .prisma_client()
        .file_handler_task()
        .find_many(vec![file_handler_task::exit_code::equals(None)])
        // 按照 id 做一下排序，保证任务下次执行的时候还是正确的顺序
        .order_by(file_handler_task::OrderByParam::Id(Direction::Asc))
        .exec()
        .await?;

    // group tasks by asset_object_id
    let mut asset_tasks: HashMap<i32, Vec<String>> = HashMap::new();
    tasks.into_iter().for_each(|task| {
        if let Some(tasks) = asset_tasks.get_mut(&task.asset_object_id) {
            tasks.push(task.task_type.to_string());
        } else {
            asset_tasks.insert(task.asset_object_id, vec![task.task_type.to_string()]);
        }
    });

    // get all asset objects
    let asset_objects = library
        .prisma_client()
        .asset_object()
        .find_many(vec![asset_object::id::in_vec(
            asset_tasks.keys().cloned().collect(),
        )])
        .exec()
        .await?;

    // trigger tasks
    for asset_object in asset_objects {
        if let Some(tasks) = asset_tasks.get(&asset_object.id).cloned() {
            create_file_handler_task(
                &asset_object,
                ctx,
                Some(tasks.iter().map(|v| v.as_str()).collect()),
                None,
            )
            .await
            .map_err(|e| anyhow::anyhow!("failed to create video task: {e:?}"))?;
        }
    }

    Ok(())
}

pub async fn create_file_handler_task(
    asset_object_data: &asset_object::Data,
    ctx: &impl CtxWithLibrary,
    task_types: Option<Vec<&str>>,
    with_existing_artifacts: Option<bool>,
) -> anyhow::Result<()> {
    let tx = ctx.task_tx()?;
    let library = ctx.library()?;

    let handler = get_file_handler(asset_object_data, ctx)?;

    let valid_task_types = handler.get_supported_task_types();
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

    for (idx, task_type) in task_types.iter().enumerate() {
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

        let priority = TaskPriority::new(task_type.1);

        match tx.send(TaskPayload::Task((
            Task {
                handler: handler.clone(),
                task_type: task_type.0.clone(),
                with_existing_artifacts,
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

pub fn get_file_handler(
    asset_object_data: &asset_object::Data,
    ctx: &dyn CtxWithLibrary,
) -> anyhow::Result<Arc<Box<dyn FileHandler>>> {
    let library = ctx.library()?;

    let handler: Box<dyn FileHandler> = match &asset_object_data.mime_type {
        Some(mime_type) => {
            if mime_type.starts_with("video") {
                let handler = VideoHandler::new(&asset_object_data.hash, &library)?;

                let library_settings = get_library_settings(&library.dir);

                let ai_handler = ctx.ai_handler()?;
                let qdrant_info: content_library::QdrantServerInfo = ctx.qdrant_info()?;

                let handler = handler
                    .with_multi_modal_embedding(
                        ai_handler.multi_modal_embedding.as_ref(),
                        &library_settings.models.multi_modal_embedding,
                        &qdrant_info.vision_collection.name,
                    )
                    .with_image_caption(
                        ai_handler.image_caption.as_ref(),
                        &library_settings.models.image_caption,
                    )
                    .with_audio_transcript(
                        ai_handler.audio_transcript.as_ref(),
                        &library_settings.models.audio_transcript,
                    )
                    .with_text_embedding(
                        ai_handler.text_embedding.as_ref(),
                        &library_settings.models.text_embedding,
                        &qdrant_info.language_collection.name,
                    );

                Box::new(handler)
            } else if mime_type.starts_with("image") {
                let handler = VideoHandler::new(&asset_object_data.hash, &library)?;

                let library_settings = get_library_settings(&library.dir);

                let ai_handler = ctx.ai_handler()?;
                let qdrant_info = ctx.qdrant_info()?;

                let handler = handler
                    .with_multi_modal_embedding(
                        ai_handler.multi_modal_embedding.as_ref(),
                        &library_settings.models.multi_modal_embedding,
                        &qdrant_info.vision_collection.name,
                    )
                    .with_image_caption(
                        ai_handler.image_caption.as_ref(),
                        &library_settings.models.image_caption,
                    )
                    .with_text_embedding(
                        ai_handler.text_embedding.as_ref(),
                        &library_settings.models.text_embedding,
                        &qdrant_info.language_collection.name,
                    );

                Box::new(handler)
            } else {
                bail!("Unsupported mime type: {:?}", &asset_object_data.mime_type)
            }
        }
        _ => {
            bail!("Unsupported mime type: {:?}", &asset_object_data.mime_type)
        }
    };

    Ok(Arc::new(handler))
}

pub fn get_file_handler_with_library(
    asset_object_data: &asset_object::Data,
    library: Arc<Library>,
    ai_handler: AIHandler,
    qdrant_info: QdrantServerInfo,
) -> anyhow::Result<Arc<Box<dyn FileHandler>>> {
    let handler: Box<dyn FileHandler> = match &asset_object_data.mime_type {
        Some(mime_type) => {
            if mime_type.starts_with("video") {
                let handler = VideoHandler::new(&asset_object_data.hash, &library)?;

                let library_settings = get_library_settings(&library.dir);

                let handler = handler
                    .with_multi_modal_embedding(
                        ai_handler.multi_modal_embedding.as_ref(),
                        &library_settings.models.multi_modal_embedding,
                        &qdrant_info.vision_collection.name,
                    )
                    .with_image_caption(
                        ai_handler.image_caption.as_ref(),
                        &library_settings.models.image_caption,
                    )
                    .with_audio_transcript(
                        ai_handler.audio_transcript.as_ref(),
                        &library_settings.models.audio_transcript,
                    )
                    .with_text_embedding(
                        ai_handler.text_embedding.as_ref(),
                        &library_settings.models.text_embedding,
                        &qdrant_info.language_collection.name,
                    );

                Box::new(handler)
            } else if mime_type.starts_with("image") {
                let handler = VideoHandler::new(&asset_object_data.hash, &library)?;

                let library_settings = get_library_settings(&library.dir);

                let handler = handler
                    .with_multi_modal_embedding(
                        ai_handler.multi_modal_embedding.as_ref(),
                        &library_settings.models.multi_modal_embedding,
                        &qdrant_info.vision_collection.name,
                    )
                    .with_image_caption(
                        ai_handler.image_caption.as_ref(),
                        &library_settings.models.image_caption,
                    )
                    .with_text_embedding(
                        ai_handler.text_embedding.as_ref(),
                        &library_settings.models.text_embedding,
                        &qdrant_info.language_collection.name,
                    );

                Box::new(handler)
            } else {
                bail!("Unsupported mime type: {:?}", &asset_object_data.mime_type)
            }
        }
        _ => {
            bail!("Unsupported mime type: {:?}", &asset_object_data.mime_type)
        }
    };

    Ok(Arc::new(handler))
}
