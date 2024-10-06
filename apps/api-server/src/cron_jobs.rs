use content_base::{delete::DeletePayload, ContentBase};
use content_library::Library;
use prisma_lib::{asset_object, file_handler_task};
use std::sync::Arc;

pub async fn delete_unlinked_assets(library: Arc<Library>, content_base: Arc<ContentBase>) {
    // 查找所有未关联 file path 的 asset object
    // TODO 怎么优化更好点
    let all_assets = match library
        .prisma_client()
        .asset_object()
        .find_many(vec![])
        .with(asset_object::file_paths::fetch(vec![]).take(1))
        .exec()
        .await
    {
        Ok(assets) => assets,
        Err(e) => {
            tracing::error!("failed to find asset_object: {}", e);
            return;
        }
    };

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
        if let Err(e) = library
            .prisma_client()
            .asset_object()
            .delete(asset_object::id::equals(asset.id))
            .exec()
            .await
        {
            tracing::error!("failed to delete asset_object: {}", e);
        }

        if let Err(e) = library
            .prisma_client()
            .file_handler_task()
            .delete_many(vec![file_handler_task::asset_object_id::equals(asset.id)])
            .exec()
            .await
        {
            tracing::error!("failed to delete asset related tasks: {}", e);
        }

        // delete from fs
        let file_path = library.file_path(&asset.hash);
        if let Err(e) = std::fs::remove_file(&file_path) {
            tracing::error!("failed to delete file({}): {}", file_path.display(), e);
        };
        let payload = DeletePayload::new(&asset.hash);
        if let Err(e) = content_base.delete(payload).await {
            tracing::error!("failed to delete artifacts: {}", e);
        }
    }
}
