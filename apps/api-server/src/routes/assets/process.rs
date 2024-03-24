use prisma_lib::file_path;
use crate::task_queue::create_video_task;
use crate::CtxWithLibrary;
use content_library::Library;

pub async fn process_video_asset(
    library: &Library,
    ctx: &impl CtxWithLibrary,
    file_path_id: i32,
) -> Result<(), rspc::Error> {
    let tx = ctx.get_task_tx();
    let file_path_data = library.prisma_client()
        .file_path()
        .find_unique(file_path::id::equals(file_path_id))
        .with(file_path::asset_object::fetch())
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

    let asset_object_data = file_path_data.asset_object
        .unwrap()
        .ok_or_else(|| {
            rspc::Error::new(
                rspc::ErrorCode::InternalServerError,
                String::from("file_path.asset_object is None"),
            )
        })?;
    // let asset_object_data = *asset_object_data;

    match create_video_task(
        &file_path_data.materialized_path,
        &asset_object_data,
        ctx,
        tx
    ).await {
        Ok(_) => Ok(()),
        Err(_) => Err(rspc::Error::new(
            rspc::ErrorCode::InternalServerError,
            String::from("failed to create video task"),
        )),
    }
}
