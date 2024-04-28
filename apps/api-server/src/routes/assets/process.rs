use crate::file_handler::create_file_handler_task;
use crate::CtxWithLibrary;
use content_library::Library;
use file_handler::video::VideoHandler;
use prisma_client_rust::QueryError;
use prisma_lib::{asset_object, file_path, media_data};
use tracing::{error, info};

pub async fn process_video_asset(
    library: &Library,
    ctx: &impl CtxWithLibrary,
    file_path_id: i32,
    with_existing_artifacts: Option<bool>,
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

    match create_file_handler_task(&asset_object_data, ctx, None, with_existing_artifacts).await {
        Ok(_) => Ok(()),
        Err(_) => Err(rspc::Error::new(
            rspc::ErrorCode::InternalServerError,
            String::from("failed to create video task"),
        )),
    }
}

pub async fn process_video_metadata(
    library: &Library,
    asset_object_id: i32,
) -> Result<(), rspc::Error> {
    info!("process video metadata for asset_object_id: {asset_object_id}");
    let sql_error = |e: QueryError| {
        error!("sql query failed: {e}",);
        rspc::Error::new(
            rspc::ErrorCode::InternalServerError,
            format!("sql query failed: {}", e),
        )
    };
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
    let video_handler = VideoHandler::new(&asset_object_data.hash, &library).map_err(|e| {
        error!("Failed to create video handler: {e}");
        rspc::Error::new(
            rspc::ErrorCode::InternalServerError,
            format!("failed to get video metadata: {}", e),
        )
    })?;
    let metadata = video_handler.get_metadata().map_err(|e| {
        error!("failed to get video metadata from video handler: {e}");
        rspc::Error::new(
            rspc::ErrorCode::InternalServerError,
            format!("failed to get video metadata: {}", e),
        )
    })?;
    video_handler.save_thumbnail(Some(0)).await.map_err(|e| {
        error!("failed to save thumbnail: {e}");
        rspc::Error::new(
            rspc::ErrorCode::InternalServerError,
            format!("failed to save thumbnail: {}", e),
        )
    })?;

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
    Ok(())
}
