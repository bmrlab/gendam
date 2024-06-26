use crate::{ai::AIHandler, file_handler::get_file_handler_with_library};
use content_library::{Library, QdrantServerInfo};
use prisma_lib::asset_object;
use std::sync::Arc;
use tracing::error;

pub async fn delete_unlinked_assets(
    library: Arc<Library>,
    ai_handler: AIHandler,
    qdrant_info: QdrantServerInfo,
) -> Result<(), rspc::Error> {
    // 查找所有未关联file path的assobject
    // todo 怎么优化更好点
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

        // delete from fs
        let file_path = library.file_path(&asset.hash);
        if let Err(e) = std::fs::remove_file(&file_path) {
            error!("failed to delete file({}): {}", file_path.display(), e);
        };
        match get_file_handler_with_library(
            &asset,
            library.clone(),
            ai_handler.clone(),
            qdrant_info.clone(),
        ) {
            Ok(handler) => {
                if let Err(e) = handler.delete_artifacts().await {
                    error!("failed to delete artifacts: {}", e);
                }
            }
            _ => {
                error!("failed to get file handler");
            }
        }
    }

    Ok(())
}
