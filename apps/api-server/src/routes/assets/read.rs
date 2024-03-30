use prisma_lib::{asset_object, file_path};
use content_library::Library;
use super::utils::normalized_materialized_path;

// #[tracing::instrument(level = "info")]
pub async fn list_file_path(
    library: &Library,
    path: &str,
    dirs_only: bool,
) -> Result<Vec<file_path::Data>, rspc::Error> {
    let materialized_path = normalized_materialized_path(path);
    let mut where_params = vec![file_path::materialized_path::equals(materialized_path)];
    if dirs_only {
        where_params.push(file_path::is_dir::equals(true));
    }
    let res = library.prisma_client()
        .file_path()
        .find_many(where_params)
        .with(
            file_path::asset_object::fetch()
            .with(asset_object::media_data::fetch())
        )
        .exec()
        .await
        .map_err(|e| {
            rspc::Error::new(
                rspc::ErrorCode::InternalServerError,
                format!("failed to list dirs: {}", e),
            )
        })?;
    Ok(res)
}

pub async fn get_file_path(
    library: &Library,
    path: &str,
    name: &str,
) -> Result<file_path::Data, rspc::Error> {
    let materialized_path = normalized_materialized_path(path);
    let res = library.prisma_client()
        .file_path()
        .find_unique(file_path::materialized_path_name(materialized_path, name.to_string()))
        .with(
            file_path::asset_object::fetch()
            .with(asset_object::media_data::fetch())
        )
        .exec()
        .await
        .map_err(|e| {
            rspc::Error::new(
                rspc::ErrorCode::InternalServerError,
                format!("sql query failed: {}", e),
            )
        })?;
    match res {
        Some(r) => Ok(r),
        None => Err(rspc::Error::new(
            rspc::ErrorCode::NotFound,
            String::from("file_path not found"),
        )),
    }
}
