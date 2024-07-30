use crate::CtxWithLibrary;
use content_base::{
    upsert::UpsertPayload, video::thumbnail::VideoThumbnailTask, ContentBase, ContentMetadata,
    ContentTask, FileInfo,
};
use content_handler::{file_metadata, video::VideoDecoder};
use content_library::Library;
use prisma_client_rust::QueryError;
use prisma_lib::{asset_object, file_handler_task, file_path, media_data};
use tracing::{error, info};

fn sql_error(e: QueryError) -> rspc::Error {
    error!("sql query failed: {e}",);
    rspc::Error::new(
        rspc::ErrorCode::InternalServerError,
        format!("sql query failed: {}", e),
    )
}

fn error_404(msg: &str) -> rspc::Error {
    error!("{}", msg);
    rspc::Error::new(rspc::ErrorCode::NotFound, String::from(msg))
}

pub async fn process_asset(
    library: &Library,
    ctx: &impl CtxWithLibrary,
    file_path_id: i32,
    _with_existing_artifacts: Option<bool>,
) -> Result<(), rspc::Error> {
    info!("process video asset for file_path_id: {file_path_id}");
    let file_path_data = library
        .prisma_client()
        .file_path()
        .find_unique(file_path::id::equals(file_path_id))
        .exec()
        .await
        .map_err(|e| {
            rspc::Error::new(
                rspc::ErrorCode::InternalServerError,
                format!("failed to find file_path: {}", e),
            )
        })?
        .ok_or_else(|| {
            rspc::Error::new(
                rspc::ErrorCode::InternalServerError,
                format!("failed to find file_path"),
            )
        })?;

    let asset_object_id = file_path_data.asset_object_id.ok_or_else(|| {
        rspc::Error::new(
            rspc::ErrorCode::InternalServerError,
            String::from("file_path.asset_object_id is None"),
        )
    })?;

    let asset_object_data = library
        .prisma_client()
        .asset_object()
        .find_unique(asset_object::id::equals(asset_object_id))
        .exec()
        .await
        .map_err(|e| {
            rspc::Error::new(
                rspc::ErrorCode::InternalServerError,
                format!("failed to find asset_object: {}", e),
            )
        })?
        .ok_or_else(|| {
            rspc::Error::new(
                rspc::ErrorCode::InternalServerError,
                format!("failed to find asset_object"),
            )
        })?;

    let cb = ctx.content_base()?;
    let payload = UpsertPayload::new(
        &asset_object_data.hash,
        library.file_path(&asset_object_data.hash),
    );
    match cb.upsert(payload).await {
        Ok(mut rx) => {
            let library = library.clone();
            // 接收来自 content_base 的任务状态通知
            // TODO 任务状态通知的实现还需要进一步优化
            tokio::spawn(async move {
                while let Some(msg) = rx.recv().await {
                    tracing::debug!("receive message: {:?}", msg);

                    let update = match msg.status {
                        content_base::TaskStatus::Started => {
                            vec![
                                file_handler_task::exit_code::set(None),
                                file_handler_task::exit_message::set(None),
                                file_handler_task::ends_at::set(None),
                                file_handler_task::starts_at::set(Some(chrono::Utc::now().into())),
                            ]
                        }
                        content_base::TaskStatus::Error => {
                            vec![
                                file_handler_task::exit_code::set(Some(1)),
                                file_handler_task::exit_message::set(msg.message),
                                file_handler_task::ends_at::set(Some(chrono::Utc::now().into())),
                            ]
                        }
                        content_base::TaskStatus::Finished => {
                            vec![
                                file_handler_task::exit_code::set(Some(0)),
                                file_handler_task::ends_at::set(Some(chrono::Utc::now().into())),
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
                        error!("Failed to update task: {}", e);
                    }
                }
            });

            Ok(())
        }
        Err(_) => Err(rspc::Error::new(
            rspc::ErrorCode::InternalServerError,
            String::from("failed to create video task"),
        )),
    }
}

#[tracing::instrument(skip(library, content_base))]
pub async fn process_asset_metadata(
    library: &Library,
    content_base: &ContentBase,
    asset_object_id: i32,
) -> Result<(), rspc::Error> {
    info!("process video metadata for asset_object_id: {asset_object_id}");
    let asset_object_data = match library
        .prisma_client()
        .asset_object()
        .find_unique(asset_object::id::equals(asset_object_id))
        .exec()
        .await
        .map_err(sql_error)?
    {
        Some(asset_object_data) => asset_object_data,
        None => {
            error!("failed to find file_path or asset_object");
            return Err(rspc::Error::new(
                rspc::ErrorCode::NotFound,
                String::from("failed to find file_path or asset_object"),
            ));
        }
    };

    let video_path = library.file_path(&asset_object_data.hash);
    let metadata = file_metadata(&video_path).map_err(|e| {
        error!("failed to get video metadata: {e}");
        rspc::Error::new(
            rspc::ErrorCode::InternalServerError,
            format!("failed to get video metadata: {}", e),
        )
    })?;

    match metadata {
        ContentMetadata::Video(metadata) => {
            if let Err(e) = VideoThumbnailTask
                .run(
                    &FileInfo {
                        file_identifier: asset_object_data.hash.clone(),
                        file_path: video_path.clone(),
                    },
                    content_base.ctx(),
                )
                .await
            {
                error!("failed to run thumbnail task: {e}");
            }

            let values: Vec<media_data::SetParam> = vec![
                media_data::width::set(Some(metadata.width as i32)),
                media_data::height::set(Some(metadata.height as i32)),
                media_data::duration::set(Some(metadata.duration as i32)),
                media_data::bit_rate::set(Some(metadata.bit_rate as i32)),
                media_data::has_audio::set(Some(metadata.audio.is_some())),
            ];
            library
                .prisma_client()
                .media_data()
                .upsert(
                    media_data::asset_object_id::equals(asset_object_data.id),
                    media_data::create(asset_object_data.id, values.clone()),
                    values.clone(),
                )
                .exec()
                .await
                .map_err(sql_error)?;
        }
        ContentMetadata::Audio(_metadata) => {
            todo!()
        }
        _ => {}
    }

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
    info!("export video segment for asset_object_id: {asset_object_id}");

    let asset_object_data = match library
        .prisma_client()
        .asset_object()
        .find_unique(asset_object::id::equals(asset_object_id))
        .exec()
        .await
        .map_err(sql_error)?
    {
        Some(asset_object_data) => asset_object_data,
        None => return Err(error_404("failed to find asset_object")),
    };
    let video_path = library.file_path(&asset_object_data.hash);

    let video_decoder = VideoDecoder::new(video_path).map_err(|e| {
        error!("Failed to create video decoder: {e}");
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
            error!("failed to save video segment: {e}");
            rspc::Error::new(
                rspc::ErrorCode::InternalServerError,
                format!("failed to save video segment: {}", e),
            )
        })?;

    Ok(())
}
