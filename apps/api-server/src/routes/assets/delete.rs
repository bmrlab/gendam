use crate::{file_handler::get_file_handler, CtxWithLibrary};
use chrono::Timelike;
use global_variable::get_current_fs_storage;
use prisma_client_rust::{Direction, QueryError};
use prisma_lib::{asset_object, file_path, trash};
use std::{collections::HashSet, sync::Arc};
use storage::prelude::*;
use tokio::sync::Mutex;
use tracing::error;

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
            let mut parent_id: Option<i32> = None;
            if materialized_path != "/" {
                let materialized_path_s = &materialized_path[..materialized_path.len()-1];
                let parts = materialized_path_s.rsplit_once("/");
                if let Some(parts) = parts {
                    tracing::debug!("parts:{parts:?}");
                    let parent_materialized_path = format!("{}/",parts.0);
                    let parent_name = parts.1;
                    tracing::debug!("parent_materialized_path: {parent_materialized_path}, parent_name:{parent_name}");
                    let parent = client
                        .file_path()
                        .find_unique(file_path::materialized_path_name(
                            parent_materialized_path.to_string(),
                            parent_name.to_string(),
                        ))
                        .exec()
                        .await
                        .map_err(|e| {
                            tracing::error!("failed to find parent item: {}", e);
                            e
                        })?;
                    if let Some(parent) = parent {
                        parent_id = Some(parent.id)
                    }
                }
            }

            let data = client
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

            let materialized_path_startswith = format!("{}{}/", &materialized_path, &name);

            // 找到所有子文件夹
            let sub_data_list = client
                .file_path()
                .find_many(vec![file_path::materialized_path::starts_with(
                    materialized_path_startswith.clone(),
                )])
                .exec()
                .await
                .map_err(|e| {
                    tracing::error!("failed to find: {}", e);
                    e
                })?;

            client
                .file_path()
                .delete_many(vec![file_path::materialized_path::starts_with(
                    materialized_path_startswith.clone(),
                )])
                .exec()
                .await
                .map_err(|e| {
                    tracing::error!("failed to delete file_path for children: {}", e);
                    e
                })?;

            // 检查trash是否有同名文件或者文件夹
            let existing = client
                .trash()
                .find_unique(trash::materialized_path_name(
                    materialized_path.to_string(),
                    name.to_string(),
                ))
                .exec()
                .await
                .map_err(|e| {
                    tracing::error!("failed to find file_path: {}", e);
                    e
                })?;
            let mut new_name = data.name.clone();
            if let Some(_existing) = existing {
                let now = chrono::Local::now();
                let timestamp =
                    format!("{:02}.{:02}.{:02}", now.hour(), now.minute(), now.second());
                new_name.push_str(&format!(" {}", timestamp));
            }
            // 创建文件
            let new_data = client
                .trash()
                .create(
                    data.is_dir,
                    "/".to_string(),
                    new_name,
                    vec![
                        trash::description::set(data.description),
                        trash::asset_object_id::set(data.asset_object_id),
                        trash::origin_parent_id::set(parent_id),
                    ],
                )
                .exec()
                .await
                .map_err(|e| {
                    tracing::error!("failed to create trash file_path: {}", e);
                    e
                })?;

            // 创建子文件和子文件夹
            for sub_data in sub_data_list {
                let data_path = format!("{}{}", data.materialized_path.clone(), data.name);
                let sub_data_path = sub_data.materialized_path.clone();
                let suffix: &str = &sub_data_path[data_path.len()..];
                let sub_materialized_path =
                    format!("{}{}{}", new_data.materialized_path, new_data.name, suffix);
                client
                    .trash()
                    .create(
                        sub_data.is_dir,
                        sub_materialized_path,
                        sub_data.name,
                        vec![
                            trash::description::set(sub_data.description),
                            trash::asset_object_id::set(sub_data.asset_object_id),
                        ],
                    )
                    .exec()
                    .await
                    .map_err(|e| {
                        tracing::error!("failed to create trash file_path: {}", e);
                        e
                    })?;
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

    Ok(())
}

pub async fn delete_trash_file_path(
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
                .trash()
                .delete(trash::materialized_path_name(
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

            // 找到所有子文件夹
            let _sub_data_list = client
                .trash()
                .find_many(vec![trash::materialized_path::starts_with(
                    materialized_path_startswith.clone(),
                )])
                .exec()
                .await
                .map_err(|e| {
                    tracing::error!("failed to find: {}", e);
                    e
                })?;

            client
                .trash()
                .delete_many(vec![trash::materialized_path::starts_with(
                    materialized_path_startswith.clone(),
                )])
                .exec()
                .await
                .map_err(|e| {
                    tracing::error!("failed to delete file_path for children: {}", e);
                    e
                })?;
            // todo 删除未链接的asset
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

#[allow(dead_code)]
pub async fn delete_file_path_and_unlinked_asset_objects(
    ctx: &dyn CtxWithLibrary,
    materialized_path: &str,
    name: &str,
) -> Result<(), rspc::Error> {
    let library = ctx.library()?;

    let deleted_asset_objects = Arc::new(Mutex::new(vec![]));
    let deleted_asset_objects_clone = deleted_asset_objects.clone();

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
                        deleted_asset_objects
                            .lock()
                            .await
                            .push(deleted_asset_object);
                        // deleted_file_hashes
                        //     .lock()
                        //     .await
                        //     .push(deleted_asset_object.hash);
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

    let storage = get_current_fs_storage!().map_err(|e| {
        rspc::Error::new(
            rspc::ErrorCode::InternalServerError,
            format!("failed to get current storage: {}", e),
        )
    })?;

    // delete from fs
    deleted_asset_objects_clone
        .lock()
        .await
        .iter()
        .for_each(|data| {
            let file_path = library.relative_file_path(&data.hash);

            if let Err(e) = storage.remove_file(file_path.clone()) {
                error!("failed to delete file({}): {}", file_path.display(), e);
            };
        });

    for data in deleted_asset_objects_clone.lock().await.iter() {
        match get_file_handler(data, ctx) {
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
