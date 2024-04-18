use crate::CtxWithLibrary;
use file_handler::delete_artifacts::handle_delete_artifacts;
use prisma_client_rust::{Direction, QueryError};
use prisma_lib::{asset_object, file_path};
use std::{collections::HashSet, sync::Arc};
use tokio::sync::Mutex;
use tracing::error;

pub async fn delete_file_path(
    ctx: &dyn CtxWithLibrary,
    materialized_path: &str,
    name: &str,
) -> Result<(), rspc::Error> {
    let library = ctx.library()?;

    let deleted_file_hashes = Arc::new(Mutex::new(vec![]));
    let deleted_file_hashes_clone = deleted_file_hashes.clone();

    library
        .prisma_client()
        ._transaction()
        .run(|client| async move {
            let mut related_asset_object_ids = HashSet::new();

            let deleted_one = client
                .file_path()
                .delete(file_path::materialized_path_name(
                    materialized_path.to_string(),
                    name.to_string(),
                ))
                .exec()
                .await
                .map_err(|e| {
                    tracing::error!("failed to delete file_path item: {}", e);
                    e
                })?;
            related_asset_object_ids.insert(deleted_one.asset_object_id);

            let materialized_path_startswith = format!("{}{}/", &materialized_path, &name);
            // TODO 这里分页查一下
            // 会比较慢
            let mut skip = 0;
            loop {
                let data: Vec<_> = client
                    .file_path()
                    .find_many(vec![file_path::materialized_path::starts_with(
                        materialized_path_startswith.clone(),
                    )])
                    .order_by(file_path::OrderByParam::Id(Direction::Asc))
                    .skip(skip)
                    .take(50)
                    .exec()
                    .await?;

                if data.len() == 0 {
                    break;
                }

                data.iter().for_each(|v| {
                    related_asset_object_ids.insert(v.asset_object_id);
                });

                skip += data.len() as i64;
            }

            // TODO 之前已经查过id了，这里也许可以优化一下
            client
                .file_path()
                .delete_many(vec![file_path::materialized_path::starts_with(
                    materialized_path_startswith,
                )])
                .exec()
                .await
                .map_err(|e| {
                    tracing::error!("failed to delete file_path for children: {}", e);
                    e
                })?;

            // TODO 这里循环删，感觉又会有点慢
            // 但是好像没有特别好的写法
            for asset_object_id in related_asset_object_ids {
                if let Some(asset_object_id) = asset_object_id {
                    let count = client
                        .file_path()
                        .count(vec![file_path::asset_object_id::equals(Some(
                            asset_object_id,
                        ))])
                        .exec()
                        .await?;
                    if count == 0 {
                        let deleted_asset_object = client
                            .asset_object()
                            .delete(asset_object::id::equals(asset_object_id))
                            .exec()
                            .await?;
                        deleted_file_hashes
                            .lock()
                            .await
                            .push(deleted_asset_object.hash);
                    }
                }
            }

            Ok(())
        })
        .await
        .map_err(|e: QueryError| {
            rspc::Error::new(
                rspc::ErrorCode::InternalServerError,
                format!("failed to delete file_path: {}", e),
            )
        })?;

    // delete from fs
    deleted_file_hashes_clone
        .lock()
        .await
        .iter()
        .for_each(|file_hash| {
            let file_path = library.file_path(file_hash);
            if let Err(e) = std::fs::remove_file(&file_path) {
                error!("failed to delete file({}): {}", file_path.display(), e);
            };
        });

    let qdrant_info = ctx.qdrant_info()?;

    handle_delete_artifacts(
        &library,
        deleted_file_hashes_clone
            .lock()
            .await
            .iter()
            .map(|v| v.to_string())
            .collect(),
        &qdrant_info.vision_collection.name,
        &qdrant_info.language_collection.name,
        true,
    )
    .await
    .map_err(|e| {
        rspc::Error::new(
            rspc::ErrorCode::InternalServerError,
            format!("failed to delete artifacts: {}", e),
        )
    })?;

    Ok(())
}
