use crate::{ai::AIHandler, file_handler::get_file_handler_with_library};
use chrono::{Duration, FixedOffset, Utc};
use content_library::{Library, QdrantServerInfo};
use prisma_lib::{asset_object, trash};
use std::sync::Arc;
use tracing::error;

pub async fn delete_unlinked_assets(
    library: Arc<Library>,
    ai_handler: AIHandler,
    qdrant_info: QdrantServerInfo,
) -> Result<(), rspc::Error> {
    // 查找所有未关联file path的assobject
    let all_assets = library
        .prisma_client()
        .asset_object()
        .find_many(vec![])
        .with(asset_object::file_paths::fetch(vec![]).take(1))
        .with(asset_object::trashes::fetch(vec![]).take(1))
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
                && asset
                    .trashes
                    .as_ref()
                    .is_some_and(|trashes| trashes.len() == 0)
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

pub async fn delete_expired_trash(library: Arc<Library>) -> Result<(), rspc::Error> {
    // 获取60天前的日期时间
    let sixty_days_ago_utc = Utc::now() - Duration::days(60);
    let fixed_offset = FixedOffset::east_opt(0);
    if let Some(fixed_offset) = fixed_offset {
        let sixty_days_ago = sixty_days_ago_utc.with_timezone(&fixed_offset);

        let trashes = library
            .prisma_client()
            .trash()
            .find_many(vec![trash::created_at::lte(sixty_days_ago)])
            .exec()
            .await
            .map_err(|e| {
                rspc::Error::new(
                    rspc::ErrorCode::InternalServerError,
                    format!("failed to find trashes: {}", e),
                )
            })?;
        tracing::debug!("expired_trashes: {trashes:?}");
        // 删除数据
        let ids = trashes.iter().map(|item| item.id).collect::<Vec<_>>();

        library
            .prisma_client()
            .trash()
            .delete_many(vec![trash::id::in_vec(ids)])
            .exec()
            .await
            .map_err(|e| {
                rspc::Error::new(
                    rspc::ErrorCode::InternalServerError,
                    format!("failed to delete trashes: {}", e),
                )
            })?;
    }
    Ok(())
}
