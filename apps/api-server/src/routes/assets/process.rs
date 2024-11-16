use crate::CtxWithLibrary;
use content_base::{delete::DeletePayload, upsert::UpsertPayload, ContentBase, TaskStatus};
use content_base_task::{
    audio::thumbnail::AudioThumbnailTask, image::thumbnail::ImageThumbnailTask,
    video::thumbnail::VideoThumbnailTask, ContentTask, FileInfo,
};
use content_handler::{file_metadata, video::VideoDecoder};
use content_library::Library;
use content_metadata::ContentMetadata;
use tracing::Instrument;

#[tracing::instrument(skip(library, ctx))]
pub async fn build_content_index(
    library: &Library,
    ctx: &impl CtxWithLibrary,
    asset_object_hash: &str, // 为了更好的 tracing, 这里用 hash 而不是用 id
    with_existing_artifacts: bool,
) -> Result<(), rspc::Error> {
    tracing::info!("building content index for");

    let asset_object_data = library
        .prisma_client()
        .asset_object()
        .find_unique(prisma_lib::asset_object::hash::equals(
            asset_object_hash.to_string(),
        ))
        .exec()
        .await?
        .ok_or_else(|| {
            rspc::Error::new(
                rspc::ErrorCode::NotFound,
                format!("failed to find asset_object"),
            )
        })?;

    library
        .prisma_client()
        .file_handler_task()
        .delete_many(vec![
            prisma_lib::file_handler_task::asset_object_id::equals(asset_object_data.id),
        ])
        .exec()
        .await?;

    let content_base = ctx.content_base()?;
    content_base
        .delete(
            DeletePayload::new(asset_object_hash)
                // 重建的时候无需删除已有索引，创建之前会自动删除 (`content_base::db::_purge_index_before_create`)，这样在处理的过程中索引依然可用
                .keep_search_indexes(true)
                // 如果 with_existing_artifacts 是 true，重建的时候无需删除已经完成的任务的 artifacts，这样任务执行的时候会字节跳过
                .keep_completed_tasks(with_existing_artifacts),
        )
        .await
        .map_err(|e| {
            rspc::Error::new(
                rspc::ErrorCode::InternalServerError,
                format!(
                    "failed to delete artifacts for {}: {}",
                    asset_object_hash, e
                ),
            )
        })?;

    tracing::debug!("asset media data: {:?}", &asset_object_data.media_data);

    let content_metadata = {
        if let Some(v) = asset_object_data.media_data.as_deref() {
            serde_json::from_str::<ContentMetadata>(v).unwrap_or_default()
        } else {
            ContentMetadata::default()
        }
    };

    let payload = UpsertPayload::new(
        &asset_object_data.hash,
        library.file_full_path_on_disk(&asset_object_data.hash),
        &content_metadata,
    );

    match content_base.upsert(payload).await {
        Ok(mut rx) => {
            let prisma_client = library.prisma_client();
            // 接收来自 content_base 的任务状态通知
            // TODO 任务状态通知的实现还需要进一步优化
            tokio::spawn(
                async move {
                    while let Some(msg) = rx.recv().await {
                        let log_msg = format!(
                            r#"TaskNotification: "{}""#,
                            msg.message.as_deref().unwrap_or("")
                        );
                        let update = match msg.status {
                            TaskStatus::Started => {
                                tracing::info!(status = ?msg.status, task_type = ?msg.task_type, "{}", &log_msg);
                                vec![
                                    prisma_lib::file_handler_task::exit_code::set(None),
                                    prisma_lib::file_handler_task::exit_message::set(None),
                                    prisma_lib::file_handler_task::ends_at::set(None),
                                    prisma_lib::file_handler_task::starts_at::set(Some(
                                        chrono::Utc::now().into(),
                                    )),
                                ]
                            }
                            TaskStatus::Error => {
                                tracing::error!(status = ?msg.status, task_type = ?msg.task_type, "{}", &log_msg);
                                vec![
                                    prisma_lib::file_handler_task::exit_code::set(Some(1)),
                                    prisma_lib::file_handler_task::exit_message::set(
                                        msg.message.clone(),
                                    ),
                                    prisma_lib::file_handler_task::ends_at::set(Some(
                                        chrono::Utc::now().into(),
                                    )),
                                ]
                            }
                            TaskStatus::Finished => {
                                tracing::info!(status = ?msg.status, task_type = ?msg.task_type, "{}", &log_msg);
                                vec![
                                    prisma_lib::file_handler_task::exit_code::set(Some(0)),
                                    prisma_lib::file_handler_task::ends_at::set(Some(
                                        chrono::Utc::now().into(),
                                    )),
                                ]
                            }
                            TaskStatus::Cancelled => {
                                tracing::warn!(status = ?msg.status, task_type = ?msg.task_type, "{}", &log_msg);
                                vec![
                                    prisma_lib::file_handler_task::exit_code::set(Some(1)),
                                    prisma_lib::file_handler_task::exit_message::set(Some(
                                        "cancelled".into(),
                                    )),
                                    prisma_lib::file_handler_task::ends_at::set(Some(
                                        chrono::Utc::now().into(),
                                    )),
                                ]
                            }
                            _ => {
                                tracing::info!(status = ?msg.status, task_type = ?msg.task_type, "{}", &log_msg);
                                vec![]
                            }
                        };

                        let x = prisma_client
                            .file_handler_task()
                            .upsert(
                                prisma_lib::file_handler_task::asset_object_id_task_type(
                                    asset_object_data.id,
                                    msg.task_type.to_string(),
                                ),
                                prisma_lib::file_handler_task::create(
                                    asset_object_data.id,
                                    msg.task_type.to_string(),
                                    vec![],
                                ),
                                update,
                            )
                            .exec()
                            .await;

                        if let Err(e) = x {
                            tracing::error!("Failed to update task: {}", e);
                        }
                    }
                }
                .instrument(tracing::Span::current()), // 把 span 信息带到 acync block 里，这样可以在 log 里看到
            );
            Ok(())
        }
        Err(_) => Err(rspc::Error::new(
            rspc::ErrorCode::InternalServerError,
            String::from("failed to create video task"),
        )),
    }
}

pub async fn generate_thumbnail(
    library: &Library,
    content_base: &ContentBase,
    file_identifier: &str,
    file_metadata: &ContentMetadata,
) -> Result<(), rspc::Error> {
    let file_info = FileInfo {
        file_identifier: file_identifier.to_string(),
        file_full_path_on_disk: library.file_full_path_on_disk(file_identifier),
    };
    let thumbnail_handle = match file_metadata {
        ContentMetadata::Video(_) => VideoThumbnailTask.run(&file_info, content_base.ctx()),
        ContentMetadata::Audio(_) => AudioThumbnailTask.run(&file_info, content_base.ctx()),
        ContentMetadata::Image(_) => ImageThumbnailTask.run(&file_info, content_base.ctx()),
        _ => Box::pin(async { Ok(()) }),
    };

    if let Err(e) = thumbnail_handle.await {
        tracing::error!("Failed to create thumbnail: {}", e);
        return Err(rspc::Error::new(
            rspc::ErrorCode::InternalServerError,
            String::from("failed to create thumbnail"),
        ));
    }

    Ok(())
}

#[tracing::instrument(skip(library, content_base))]
pub async fn process_asset_metadata(
    library: &Library,
    content_base: &ContentBase,
    asset_object_hash: &str, // 为了更好的 tracing, 这里用 hash 而不是用 id
) -> Result<(), rspc::Error> {
    let asset_object_data = match library
        .prisma_client()
        .asset_object()
        .find_unique(prisma_lib::asset_object::hash::equals(
            asset_object_hash.to_string(),
        ))
        .exec()
        .await?
    {
        Some(asset_object_data) => asset_object_data,
        None => {
            tracing::error!("failed to find file_path or asset_object");
            return Err(rspc::Error::new(
                rspc::ErrorCode::NotFound,
                String::from("failed to find file_path or asset_object"),
            ));
        }
    };

    tracing::info!(hash = &asset_object_data.hash, "processing asset metadata");

    let file_full_path_on_disk = library.file_full_path_on_disk(&asset_object_data.hash);
    let file_extension = file_full_path_on_disk
        .extension()
        .map(|v| v.to_string_lossy().to_string());

    let (metadata, mime) = file_metadata(file_full_path_on_disk, file_extension.as_deref());

    let metadata_json = match serde_json::to_string(&metadata) {
        Ok(metadata) => Some(metadata),
        _ => None,
    };

    let prisma_client = library.prisma_client();
    let prisma_handle = prisma_client
        .asset_object()
        .update(
            prisma_lib::asset_object::id::equals(asset_object_data.id),
            vec![
                prisma_lib::asset_object::media_data::set(metadata_json),
                prisma_lib::asset_object::mime_type::set(Some(mime)),
            ],
        )
        .exec();

    // generate thumbnail
    tracing::debug!("generate thumbnail");
    let thumbnail_handle =
        generate_thumbnail(library, content_base, &asset_object_data.hash, &metadata);

    let results = tokio::join!(prisma_handle, thumbnail_handle);
    let _ = results.0?;
    let _ = results.1?;

    Ok(())
}

pub async fn export_video_segment(
    library: &Library,
    verbose_file_name: String,
    output_dir: String,
    asset_object_id: i32,
    milliseconds_from: u32,
    milliseconds_to: u32,
) -> Result<(), rspc::Error> {
    tracing::info!("export video segment for asset_object_id: {asset_object_id}");

    let asset_object_data = library
        .prisma_client()
        .asset_object()
        .find_unique(prisma_lib::asset_object::id::equals(asset_object_id))
        .exec()
        .await?
        .ok_or_else(|| {
            rspc::Error::new(
                rspc::ErrorCode::NotFound,
                format!("failed to find asset_object"),
            )
        })?;
    let video_path = library.file_full_path_on_disk(&asset_object_data.hash);

    let video_decoder = VideoDecoder::new(video_path).map_err(|e| {
        tracing::error!("Failed to create video decoder: {e}");
        rspc::Error::new(
            rspc::ErrorCode::InternalServerError,
            format!("failed to get video decoder: {}", e),
        )
    })?;

    video_decoder
        .save_video_segment(
            verbose_file_name.as_ref(),
            output_dir,
            milliseconds_from,
            milliseconds_to,
        )
        .map_err(|e| {
            tracing::error!("failed to save video segment: {e}");
            rspc::Error::new(
                rspc::ErrorCode::InternalServerError,
                format!("failed to save video segment: {}", e),
            )
        })?;

    Ok(())
}
