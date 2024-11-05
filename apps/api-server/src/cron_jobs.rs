use content_base::{delete::DeletePayload, ContentBase};
use content_library::Library;
use prisma_lib::{asset_object, file_handler_task};
use std::{path::PathBuf, sync::Arc};
use storage::Storage;

async fn remove_shared_dir(dir: PathBuf) -> anyhow::Result<()> {
    let fs_storage = global_variable::get_current_fs_storage!()?;
    tracing::debug!("remove dir or file {:?}", dir);
    fs_storage.remove_dir_all(dir.clone()).await?;
    let op = fs_storage.op()?;
    if let Some(shard_dir) = dir.parent() {
        let sub_dirs = fs_storage.read_dir(shard_dir.to_path_buf()).await?;
        let mut is_empty = true;
        for sub_dir in sub_dirs {
            let x = op.stat(sub_dir.to_string_lossy().as_ref()).await?;
            if x.is_dir() {
                is_empty = false;
                break;
            }
        }
        if is_empty {
            tracing::debug!("remove empty shard dir {:?}", shard_dir);
            fs_storage.remove_dir_all(shard_dir.to_path_buf()).await?;
        }
    }

    Ok(())
}

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

    for asset in unlinked_assets {
        tracing::debug!("delete unlinked asset {:?}", asset.hash);
        if let Err(e) = library
            .prisma_client()
            .asset_object()
            .delete(asset_object::id::equals(asset.id))
            .exec()
            .await
        {
            tracing::error!("failed to delete asset {} in db: {} ", &asset.hash, e);
            continue;
        }

        if let Err(e) = library
            .prisma_client()
            .file_handler_task()
            .delete_many(vec![file_handler_task::asset_object_id::equals(asset.id)])
            .exec()
            .await
        {
            tracing::error!("failed to task log of asset {} in db: {}", &asset.hash, e);
            continue;
        }

        let delete_payload = DeletePayload::new(&asset.hash);
        if let Err(e) = content_base.delete_search_indexes(delete_payload).await {
            tracing::error!("failed to delete search indexes of {}: {}", asset.hash, e);
            continue;
        }

        let delete_payload = DeletePayload::new(&asset.hash);
        if let Err(e) = content_base.delete_artifacts(delete_payload).await {
            tracing::error!("failed to delete artifacts for {}: {}", asset.hash, e);
            continue;
        };

        // 在没有错误地删除了任务记录后，
        // 删除 artifacts 目录中留下的的 thumbnail 和 artifacts.json 文件，以及 artifacts 目录
        let artifacts_dir = library.relative_artifacts_dir(&asset.hash);
        if let Err(e) = remove_shared_dir(artifacts_dir).await {
            tracing::error!("failed to remove artifacts dir of {}: {}", asset.hash, e);
            continue;
        }

        let file_path = library.relative_file_dir(&asset.hash);
        if let Err(e) = remove_shared_dir(file_path).await {
            tracing::error!("failed to remove file of {}: {}", asset.hash, e);
        }
    }
}
