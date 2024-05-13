use prisma_client_rust::QueryError;
use prisma_lib::{
    file_path::{self, Data},
    media_data, PrismaClient,
};
use std::sync::Arc;

pub async fn get_ids(
    prisma_client: Arc<PrismaClient>,
    dir: String,
) -> Result<(Vec<String>, Vec<String>, Vec<String>), QueryError> {
    let ids = get_file_path_ids_under_dir(prisma_client.clone(), dir).await?;

    get_id_combination(prisma_client, ids).await
}

/// Get all file path ids under a directory
/// dir: ${materialized_path}{name}
/// the result include dir type file path id
async fn get_file_path_ids_under_dir(
    prisma_client: Arc<PrismaClient>,
    dir: String,
) -> Result<Vec<String>, QueryError> {
    let query_res: Vec<Data> = prisma_client
        .file_path()
        .find_many(vec![file_path::WhereParam::MaterializedPath(
            prisma_lib::read_filters::StringFilter::StartsWith(dir),
        )])
        .exec()
        .await?;
    let ids: Vec<String> = query_res.iter().map(|d| d.id.clone()).collect();
    Ok(ids)
}

/// res: (AssetObjectIds, FilePathIds, MediaDataIds)
/// id: file path ids
async fn get_id_combination(
    prisma_client: Arc<PrismaClient>,
    ids: Vec<String>,
) -> Result<(Vec<String>, Vec<String>, Vec<String>), QueryError> {
    let file_path_vec = prisma_client
        .file_path()
        .find_many(vec![file_path::WhereParam::Id(
            prisma_lib::read_filters::StringFilter::InVec(ids.clone()),
        )])
        .exec()
        .await?;

    let asset_object_ids = file_path_vec
        .iter()
        .filter_map(|f| f.asset_object_id.clone())
        .collect::<Vec<String>>();

    let media_data_vec = prisma_client
        .media_data()
        .find_many(vec![media_data::WhereParam::AssetObjectId(
            prisma_lib::read_filters::StringNullableFilter::InVec(asset_object_ids.clone()),
        )])
        .exec()
        .await?;

    let media_data_ids = media_data_vec
        .iter()
        .map(|m| m.id.clone())
        .collect::<Vec<String>>();

    Ok((ids, asset_object_ids, media_data_ids))
}
