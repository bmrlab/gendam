use content_base::{delete::DeletePayload, ContentBase};
use content_library::Library;
use prisma_lib::{asset_object, file_handler_task};
use std::sync::Arc;
use tracing::error;

pub async fn delete_unlinked_assets(
    library: Arc<Library>,
    content_base: Arc<ContentBase>,
) -> Result<(), rspc::Error> {
    // 查找所有未关联 file path 的 asset object
    // TODO 怎么优化更好点
    let all_assets = library
        .prisma_client()
        .asset_object()
        .find_many(vec![])
        .with(asset_object::file_paths::fetch(vec![]).take(1))
        .exec()
        .await
        .map_err(|e| {
            rspc::Error::new(
                rspc::ErrorCode::InternalServerError,
                format!("failed to find asset: {}", e),
            )
        })?;

    let unlinked_assets = all_assets
        .into_iter()
        .filter(|asset| {
            asset
                .file_paths
                .as_ref()
                .is_some_and(|paths| paths.len() == 0)
        })
        .collect::<Vec<_>>();

    tracing::debug!("unlinked_assets : {:?}", unlinked_assets);

    for asset in unlinked_assets {
        library
            .prisma_client()
            .asset_object()
            .delete(asset_object::id::equals(asset.id))
            .exec()
            .await
            .map_err(|e| {
                rspc::Error::new(
                    rspc::ErrorCode::InternalServerError,
                    format!("failed to find asset: {}", e),
                )
            })?;

        library
            .prisma_client()
            .file_handler_task()
            .delete_many(vec![file_handler_task::asset_object_id::equals(asset.id)])
            .exec()
            .await
            .map_err(|e| {
                rspc::Error::new(
                    rspc::ErrorCode::InternalServerError,
                    format!("failed to delete asset related tasks: {}", e),
                )
            })?;

        // delete from fs
        let file_path = library.file_path(&asset.hash);
        if let Err(e) = std::fs::remove_file(&file_path) {
            error!("failed to delete file({}): {}", file_path.display(), e);
        };
        let payload = DeletePayload::new(&asset.hash);
        if let Err(e) = content_base.delete(payload).await {
            error!("failed to delete artifacts: {}", e);
        }
    }

    Ok(())
}
