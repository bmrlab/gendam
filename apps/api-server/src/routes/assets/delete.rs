use prisma_lib::file_path;
use prisma_client_rust::QueryError;
use content_library::Library;
use super::utils::normalized_materialized_path;

/**
 * TODO: 删除 file_path 以后要检查一下 assetobject 是否还有其他引用，如果没有的话，要删除 assetobject，进一步的，删除 filehandlertask
 */
pub async fn delete_file_path(
    library: &Library,
    path: &str,
    name: &str,
) -> Result<(), rspc::Error> {
    let materialized_path = normalized_materialized_path(path);
    let name = name.to_string();

    library.prisma_client()
        ._transaction()
        .run(|client| async move {
            client
                .file_path()
                .delete(file_path::materialized_path_name(materialized_path.clone(), name.clone()))
                .exec()
                .await
                .map_err(|e| {
                    tracing::error!("failed to delete file_path item: {}", e);
                    e
                })?;
            let materialized_path_startswith = format!("{}{}/", &materialized_path, &name);
            client
                .file_path()
                .delete_many(vec![
                    file_path::materialized_path::starts_with(materialized_path_startswith)
                ])
                .exec()
                .await
                .map_err(|e| {
                    tracing::error!("failed to delete file_path for children: {}", e);
                    e
                })?;
            Ok(())
        })
        .await
        .map_err(|e: QueryError| {
            rspc::Error::new(
                rspc::ErrorCode::InternalServerError,
                format!("failed to delete file_path: {}", e),
            )
        })?;

    Ok(())
}
