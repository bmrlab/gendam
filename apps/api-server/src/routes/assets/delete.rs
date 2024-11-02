use crate::CtxWithLibrary;
use prisma_client_rust::QueryError;

pub async fn delete_file_path(
    ctx: &dyn CtxWithLibrary,
    materialized_path: &str,
    name: &str,
) -> Result<(), rspc::Error> {
    let library = ctx.library()?;

    library
        .prisma_client()
        ._transaction()
        .run(|client| async move {
            client
                .file_path()
                .delete(prisma_lib::file_path::materialized_path_name(
                    materialized_path.to_string(),
                    name.to_string(),
                ))
                .exec()
                .await
                .map_err(|e| {
                    tracing::error!("failed to delete file_path item: {}", e);
                    e
                })?;

            let materialized_path_startswith = format!("{}{}/", &materialized_path, &name);
            client
                .file_path()
                .delete_many(vec![prisma_lib::file_path::materialized_path::starts_with(
                    materialized_path_startswith,
                )])
                .exec()
                .await
                .map_err(|e| {
                    tracing::error!("failed to delete file_path for children: {}", e);
                    e
                })?;

            Ok(()) as Result<(), QueryError>
        })
        .await?;

    Ok(())
}
