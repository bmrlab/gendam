use crate::CtxWithLibrary;
use chrono::Timelike;
use prisma_client_rust::QueryError;
use prisma_lib::{file_path, trash};

pub async fn put_back(
    ctx: &dyn CtxWithLibrary,
    materialized_path: &str,
    name: &str,
) -> Result<(), rspc::Error> {
    let library = ctx.library()?;

    // 查询需要移回的数据
    let data: Option<trash::Data> = library
        .prisma_client()
        .trash()
        .find_unique(trash::materialized_path_name(
            materialized_path.to_string(),
            name.to_string(),
        ))
        .exec()
        .await
        .map_err(|e| {
            tracing::error!("failed to delete file_path item: {}", e);
            e
        })?;

    if let Some(data) = data {
        let origin_parent_id = data.origin_parent_id;

        let mut materialized_path = "/".to_string();

        if let Some(origin_parent_id) = origin_parent_id {
            let file_path_data = library
                .prisma_client()
                .file_path()
                .find_unique(file_path::UniqueWhereParam::IdEquals(origin_parent_id))
                .exec()
                .await
                .map_err(|e| {
                    tracing::error!("failed to delete file_path item: {}", e);
                    e
                })?;

            if let Some(file_path_data) = file_path_data {
                materialized_path = format!(
                    "{}{}/",
                    file_path_data.materialized_path, file_path_data.name
                )
            }
        }

        library
            .prisma_client()
            ._transaction()
            .run(|client| async move {
                // 删除 trash 数据
                client
                    .trash()
                    .delete(trash::UniqueWhereParam::IdEquals(data.id))
                    .exec()
                    .await
                    .map_err(|e| {
                        tracing::error!("failed to delete trash item: {}", e);
                        e
                    })?;

                let materialized_path_startswith =
                    format!("{}{}/", &data.materialized_path, &data.name);

                // 找到下面所有子文件夹
                let sub_data_list = client
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

                // 检查filepath 目标路径是否有同名文件或者文件夹
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
                    .file_path()
                    .create(
                        data.is_dir,
                        materialized_path,
                        new_name,
                        vec![
                            file_path::description::set(data.description),
                            file_path::asset_object_id::set(data.asset_object_id),
                        ],
                    )
                    .exec()
                    .await
                    .map_err(|e| {
                        tracing::error!("failed to create file_path: {}", e);
                        e
                    })?;
                // 创建子文件和子文件夹
                for sub_data in sub_data_list {
                    let data_path = format!("{}{}", data.materialized_path.clone(), data.name);
                    let sub_data_path = sub_data.materialized_path.clone();
                    let suffix: &str = &sub_data_path[data_path.len()..];
                    let sub_materialized_path =
                        format!("{}{}{}", new_data.materialized_path, new_data.name, suffix);
                    tracing::info!("sub_materialized_path: {sub_materialized_path}");
                    client
                        .file_path()
                        .create(
                            sub_data.is_dir,
                            sub_materialized_path,
                            sub_data.name,
                            vec![
                                file_path::description::set(sub_data.description),
                                file_path::asset_object_id::set(sub_data.asset_object_id),
                            ],
                        )
                        .exec()
                        .await
                        .map_err(|e| {
                            tracing::error!("failed to create file_path: {}", e);
                            e
                        })?;
                }
                Ok(())
            })
            .await
            .map_err(|e: QueryError| {
                rspc::Error::new(
                    rspc::ErrorCode::InternalServerError,
                    format!("failed to put back: {}", e),
                )
            })?;
    }
    Ok(())
}
