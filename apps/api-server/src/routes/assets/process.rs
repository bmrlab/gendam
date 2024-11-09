use crate::CtxWithLibrary;
use content_base::{upsert::UpsertPayload, ContentBase};
use content_base_task::{
    audio::thumbnail::AudioThumbnailTask, image::thumbnail::ImageThumbnailTask,
    video::thumbnail::VideoThumbnailTask, ContentTask, FileInfo,
};
use content_handler::{file_metadata, video::VideoDecoder};
use content_library::Library;
use content_metadata::ContentMetadata;
use prisma_lib::{asset_object, file_handler_task};
use tracing::Instrument;

#[tracing::instrument(skip(library, ctx, _with_existing_artifacts))]
pub async fn process_asset(
    library: &Library,
    ctx: &impl CtxWithLibrary,
    asset_object_hash: String, // 为了更好的 tracing, 这里用 hash 而不是用 id
    _with_existing_artifacts: Option<bool>,
) -> Result<(), rspc::Error> {
    tracing::info!("processing asset");

    let asset_object_data = library
        .prisma_client()
        .asset_object()
        .find_unique(asset_object::hash::equals(asset_object_hash))
        .exec()
        .await?
        .ok_or_else(|| {
            rspc::Error::new(
                rspc::ErrorCode::NotFound,
                format!("failed to find asset_object"),
            )
        })?;

    tracing::debug!("asset media data: {:?}", &asset_object_data.media_data);

    let content_metadata = {
        if let Some(v) = asset_object_data.media_data {
            serde_json::from_str::<ContentMetadata>(&v).unwrap_or_default()
        } else {
            ContentMetadata::default()
        }
    };

    let cb = ctx.content_base()?;
    let payload = UpsertPayload::new(
        &asset_object_data.hash,
        library.file_full_path_on_disk(&asset_object_data.hash),
        &content_metadata,
    );
    match cb.upsert(payload).await {
        Ok(mut rx) => {
            let library = library.clone();
            // 接收来自 content_base 的任务状态通知
            // TODO 任务状态通知的实现还需要进一步优化
            tokio::spawn(
                async move {
                    while let Some(msg) = rx.recv().await {
                        tracing::info!("{:?}", msg);

                        let update = match msg.status {
                            content_base::TaskStatus::Started => {
                                vec![
                                    file_handler_task::exit_code::set(None),
                                    file_handler_task::exit_message::set(None),
                                    file_handler_task::ends_at::set(None),
                                    file_handler_task::starts_at::set(Some(
                                        chrono::Utc::now().into(),
                                    )),
                                ]
                            }
                            content_base::TaskStatus::Error => {
                                vec![
                                    file_handler_task::exit_code::set(Some(1)),
                                    file_handler_task::exit_message::set(msg.message),
                                    file_handler_task::ends_at::set(Some(
                                        chrono::Utc::now().into(),
                                    )),
                                ]
                            }
                            content_base::TaskStatus::Finished => {
                                vec![
                                    file_handler_task::exit_code::set(Some(0)),
                                    file_handler_task::ends_at::set(Some(
                                        chrono::Utc::now().into(),
                                    )),
                                ]
                            }
                            content_base::TaskStatus::Cancelled => {
                                vec![
                                    file_handler_task::exit_code::set(Some(1)),
                                    file_handler_task::exit_message::set(Some("cancelled".into())),
                                    file_handler_task::ends_at::set(Some(
                                        chrono::Utc::now().into(),
                                    )),
                                ]
                            }
                            _ => {
                                vec![]
                            }
                        };

                        let x = library
                            .prisma_client()
                            .file_handler_task()
                            .upsert(
                                file_handler_task::asset_object_id_task_type(
                                    asset_object_data.id,
                                    msg.task_type.to_string(),
                                ),
                                file_handler_task::create(
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
    asset_object_id: i32,
) -> Result<(), rspc::Error> {
    let asset_object_data = match library
        .prisma_client()
        .asset_object()
        .find_unique(asset_object::id::equals(asset_object_id))
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
            asset_object::id::equals(asset_object_data.id),
            vec![
                asset_object::media_data::set(metadata_json),
                asset_object::mime_type::set(Some(mime)),
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
        .find_unique(asset_object::id::equals(asset_object_id))
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
