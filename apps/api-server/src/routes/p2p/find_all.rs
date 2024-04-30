use prisma_lib::{file_path, PrismaClient};
use std::{collections::HashSet, sync::Arc};

pub async fn find_all_asset_object_hashes(
    file_id_list: Vec<i32>,
    prisma_client: Arc<PrismaClient>,
) -> Result<Vec<String>, rspc::Error> {
    let mut files = file_id_list.clone();
    let mut results = HashSet::new();

    loop {
        if files.is_empty() {
            break;
        }

        let file_paths_result = prisma_client
            .file_path()
            .find_many(vec![file_path::id::in_vec(files.clone())])
            .with(file_path::asset_object::fetch())
            .exec()
            .await;

        // 清空files
        files = Vec::new();

        let file_paths = match file_paths_result {
            Ok(paths) => paths,
            Err(error) => {
                tracing::error!("file_paths error: {error}");
                return Err(rspc::Error::new(
                    rspc::ErrorCode::InternalServerError,
                    format!("error: {error}"),
                ));
            }
        };

        for file_path in file_paths {
            match file_path.asset_object {
                Some(asset_object) => {
                    match asset_object {
                        Some(asset_object) => {
                            // 是文件
                            results.insert(asset_object.hash.clone());
                        }
                        None => {
                            // 是文件夹
                            let materialized_path =
                                format!("{}{}/", file_path.materialized_path, file_path.name);

                            let sub_file_res = prisma_client
                                .file_path()
                                .find_many(vec![file_path::materialized_path::equals(
                                    materialized_path.clone(),
                                )])
                                .exec()
                                .await?;
                            let res: Vec<i32> =
                                sub_file_res.iter().map(|file| file.id.clone()).collect();

                            files.extend(res);
                        }
                    }
                }
                None => {
                    // 是文件夹
                    let materialized_path =
                        format!("{}{}/", file_path.materialized_path, file_path.name);

                    let sub_file_res = prisma_client
                        .file_path()
                        .find_many(vec![file_path::materialized_path::equals(
                            materialized_path.clone(),
                        )])
                        .exec()
                        .await?;
                    let res: Vec<i32> = sub_file_res.iter().map(|file| file.id.clone()).collect();

                    files.extend(res);
                }
            }
        }
    }

    Ok(results.into_iter().collect())
}
