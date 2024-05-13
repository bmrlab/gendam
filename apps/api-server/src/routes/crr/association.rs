use prisma_client_rust::QueryError;
use prisma_lib::{
    asset_object,
    file_path::{self, Data},
    media_data, PrismaClient,
};
use std::{path::PathBuf, sync::Arc};
use tracing::info;

pub async fn get_file_path_ids_under_materialized_path(
    prisma_client: Arc<PrismaClient>,
    materialized_path: String,
) -> Result<Vec<String>, QueryError> {
    let query_res: Vec<Data> = prisma_client
        .file_path()
        .find_many(vec![file_path::WhereParam::MaterializedPath(
            prisma_lib::read_filters::StringFilter::StartsWith(materialized_path),
        )])
        .exec()
        .await?;
    let ids: Vec<String> = query_res.iter().map(|d| d.id.clone()).collect();
    Ok(ids)
}
