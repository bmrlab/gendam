use prisma_lib::{asset_object, file_path};
use content_library::Library;
use super::utils::normalized_materialized_path;
use super::types::{MediaDataQueryResult, AssetObjectQueryResult, FilePathQueryResult};

pub async fn list_file_path(
    library: &Library,
    path: &str,
    dirs_only: bool,
) -> Result<Vec<FilePathQueryResult>, rspc::Error> {
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
    // TODO 这里写的有点挫
    let res = res
        .iter()
        .map(|r| FilePathQueryResult {
            id: r.id,
            name: r.name.clone(),
            materialized_path: r.materialized_path.clone(),
            is_dir: r.is_dir,
            asset_object: match r.asset_object.as_ref() {
                Some(asset_object_data) => match asset_object_data {
                    Some(asset_object_data) => Some(AssetObjectQueryResult {
                        id: asset_object_data.id,
                        hash: asset_object_data.hash.clone(),
                        media_data: match asset_object_data.media_data {
                            Some(ref media_data) => {
                                match media_data {
                                    Some(ref media_data) => Some(MediaDataQueryResult {
                                        id: media_data.id,
                                        width: media_data.width.unwrap_or_default(),
                                        height: media_data.height.unwrap_or_default(),
                                        duration: media_data.duration.unwrap_or_default(),
                                        bit_rate: media_data.bit_rate.unwrap_or_default(),
                                        size: media_data.size.unwrap_or_default(),
                                    }),
                                    None => None,
                                }
                            },
                            None => None,
                        },
                    }),
                    None => None,
                },
                None => None,
            },
            created_at: r.created_at.to_string(),
            updated_at: r.updated_at.to_string(),
        })
        .collect::<Vec<FilePathQueryResult>>();

    Ok(res)
}

pub async fn get_file_path(
    library: &Library,
    path: &str,
    name: &str,
) -> Result<FilePathQueryResult, rspc::Error> {
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
        Some(r) => Ok(FilePathQueryResult {
            id: r.id,
            name: r.name.clone(),
            materialized_path: r.materialized_path.clone(),
            is_dir: r.is_dir,
            asset_object: match r.asset_object.as_ref() {
                Some(asset_object_data) => match asset_object_data {
                    Some(asset_object_data) => Some(AssetObjectQueryResult {
                        id: asset_object_data.id,
                        hash: asset_object_data.hash.clone(),
                        media_data: match asset_object_data.media_data {
                            Some(ref media_data) => {
                                match media_data {
                                    Some(ref media_data) => Some(MediaDataQueryResult {
                                        id: media_data.id,
                                        width: media_data.width.unwrap_or_default(),
                                        height: media_data.height.unwrap_or_default(),
                                        duration: media_data.duration.unwrap_or_default(),
                                        bit_rate: media_data.bit_rate.unwrap_or_default(),
                                        size: media_data.size.unwrap_or_default(),
                                    }),
                                    None => None,
                                }
                            },
                            None => None,
                        },
                    }),
                    None => None,
                },
                None => None,
            },
            created_at: r.created_at.to_string(),
            updated_at: r.updated_at.to_string(),
        }),
        None => Err(rspc::Error::new(
            rspc::ErrorCode::NotFound,
            String::from("file_path not found"),
        )),
    }
}
