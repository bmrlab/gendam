use content_library::Library;
use prisma_lib::{asset_object, file_path, trash};

// #[tracing::instrument(level = "info")]
pub async fn list_file_path(
    library: &Library,
    materialized_path: &str,
    is_dir: Option<bool>,
    include_sub_dirs: Option<bool>,
) -> Result<Vec<file_path::Data>, rspc::Error> {
    let mut where_params = vec![];
    if include_sub_dirs.unwrap_or(false) {
        where_params.push(file_path::materialized_path::starts_with(
            materialized_path.to_string(),
        ));
    } else {
        where_params.push(file_path::materialized_path::equals(
            materialized_path.to_string(),
        ));
    }
    if let Some(is_dir) = is_dir {
        where_params.push(file_path::is_dir::equals(is_dir));
    }
    let res = library
        .prisma_client()
        .file_path()
        .find_many(where_params)
        .with(file_path::asset_object::fetch().with(asset_object::media_data::fetch()))
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

pub async fn list_trash_file_path(
    library: &Library,
    materialized_path: &str,
    is_dir: Option<bool>,
    include_sub_dirs: Option<bool>,
) -> Result<Vec<trash::Data>, rspc::Error> {
    let mut where_params = vec![];
    if include_sub_dirs.unwrap_or(false) {
        where_params.push(trash::materialized_path::starts_with(
            materialized_path.to_string(),
        ));
    } else {
        where_params.push(trash::materialized_path::equals(
            materialized_path.to_string(),
        ));
    }
    if let Some(is_dir) = is_dir {
        where_params.push(trash::is_dir::equals(is_dir));
    }
    let res = library
        .prisma_client()
        .trash()
        .find_many(where_params)
        .with(trash::asset_object::fetch().with(asset_object::media_data::fetch()))
        .exec()
        .await
        .map_err(|e| {
            rspc::Error::new(
                rspc::ErrorCode::InternalServerError,
                format!("failed to trash list dirs: {}", e),
            )
        })?;
    Ok(res)
}

pub async fn get_file_path(
    library: &Library,
    materialized_path: &str,
    name: &str,
) -> Result<file_path::Data, rspc::Error> {
    let res = library
        .prisma_client()
        .file_path()
        .find_unique(file_path::materialized_path_name(
            materialized_path.to_string(),
            name.to_string(),
        ))
        .with(file_path::asset_object::fetch().with(asset_object::media_data::fetch()))
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
