use content_library::Library;
use prisma_client_rust::PrismaValue;
use prisma_lib::{
    asset_object,
    file_path::{self},
};
use prisma_lib::{raw, read_filters};
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Debug, Deserialize, Serialize)]
struct PathFilterResult {
    id: String,
}

// #[tracing::instrument(level = "info")]
pub async fn list_file_path(
    library: &Library,
    materialized_path: &str,
    is_dir: Option<bool>,
    include_sub_dirs: Option<bool>,
) -> Result<Vec<file_path::Data>, rspc::Error> {
    let exclude_sub_dirs_sql = if let Some(is_dir) = is_dir {
        raw!(
            "SELECT
                id
            FROM
                FilePath
            WHERE
                COALESCE(rtrim(relativePath, '/'), '') || materializedPath = {} AND isDir = {};",
            PrismaValue::String(materialized_path.to_string()),
            PrismaValue::Int(if is_dir { 1 } else { 0 })
        )
    } else {
        raw!(
            "SELECT
                id
            FROM
                FilePath
            WHERE
                COALESCE(rtrim(relativePath, '/'), '') || materializedPath = {};",
            PrismaValue::String(materialized_path.to_string())
        )
    };

    let include_sub_dirs_sql = if let Some(is_dir) = is_dir {
        raw!(
            "SELECT
                id
            FROM
                FilePath
            WHERE
                COALESCE(rtrim(relativePath, '/'), '') || materializedPath LIKE {}% AND isDir = {};",
            PrismaValue::String(materialized_path.to_string()),
            PrismaValue::Int(if is_dir { 1 } else { 0 })
        )
    } else {
        raw!(
            "SELECT
                id
            FROM
                FilePath
            WHERE
                COALESCE(rtrim(relativePath, '/'), '') || materializedPath LIKE {}%;",
            PrismaValue::String(materialized_path.to_string())
        )
    };

    let file_path_ids: Vec<PathFilterResult> = library
        .prisma_client()
        ._query_raw(if include_sub_dirs.unwrap_or(false) {
            include_sub_dirs_sql
        } else {
            exclude_sub_dirs_sql
        })
        .exec()
        .await
        .expect("failed to query file path");

    info!("file_path_ids: {:?}", file_path_ids);

    let mut res = library
        .prisma_client()
        .file_path()
        .find_many(vec![file_path::WhereParam::Id(
            read_filters::StringFilter::InVec(file_path_ids.iter().map(|f| f.id.clone()).collect()),
        )])
        .with(file_path::asset_object::fetch().with(asset_object::media_data::fetch(vec![])))
        .exec()
        .await
        .map_err(|e| {
            rspc::Error::new(
                rspc::ErrorCode::InternalServerError,
                format!("failed to list dirs: {}", e),
            )
        })?;

    res.iter_mut().for_each(|r| {
        if !r.relative_path.is_none() {
            let relative_path = r.relative_path.clone().unwrap();
            r.materialized_path = format!(
                "{}{}",
                &relative_path.strip_suffix('/').unwrap_or(&relative_path),
                r.materialized_path
            );
        }
    });

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
        .find_first(vec![
            file_path::materialized_path::equals(materialized_path.to_string()),
            file_path::name::equals(name.to_string()),
        ])
        .with(file_path::asset_object::fetch().with(asset_object::media_data::fetch(vec![])))
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
