use std::path::PathBuf;

use crate::sync::folder::create_folder_crdt;

use super::utils::generate_file_hash;
use content_library::Library;
use prisma_client_rust::QueryError;
use prisma_lib::{asset_object, file_path};

pub async fn create_dir(
    library: &Library,
    materialized_path: &str,
    name: &str,
) -> Result<file_path::Data, rspc::Error> {
    /*
     * TODO
     * 如果 path 是 /a/b/c/, 要确保存在一条数据 {path:"/a/b/",name:"c"}, 不然就是文件夹不存在
     */
    let res = library
        .prisma_client()
        .file_path()
        .create(
            true,
            materialized_path.to_string(),
            name.to_string(),
            vec![],
        )
        .exec()
        .await
        .map_err(|e| {
            rspc::Error::new(
                rspc::ErrorCode::InternalServerError,
                format!("failed to create file_path: {}", e),
            )
        })?;
    // 触发创建文件夹的crdt
    create_folder_crdt(&library, materialized_path.to_owned(), name.to_owned())
        .await
        .expect("failed to create crdt");
    Ok(res)
}

pub async fn create_asset_object(
    library: &Library,
    materialized_path: &str,
    name: &str,
    local_full_path: &str,
) -> Result<(file_path::Data, asset_object::Data, bool), rspc::Error> {
    let start_time = std::time::Instant::now();
    let fs_metadata = std::fs::metadata(&local_full_path).map_err(|e| {
        rspc::Error::new(
            rspc::ErrorCode::InternalServerError,
            format!("failed to get video metadata: {}", e),
        )
    })?;
    let kind = infer::get_from_path(&local_full_path);
    let file_mime_type = match kind {
        Ok(Some(mime)) => Some(mime.to_string()),
        _ => None,
    };
    let file_size_in_bytes = fs_metadata.len() as i32;
    let file_hash = generate_file_hash(&local_full_path, fs_metadata.len() as u64)
        .await
        .map_err(|e| {
            rspc::Error::new(
                rspc::ErrorCode::InternalServerError,
                format!("failed to generate file hash: {}", e),
            )
        })?;
    let duration = start_time.elapsed();
    tracing::info!(
        "{:?}, hash: {:?}, mime_type: {:?}, duration: {:?}",
        local_full_path,
        file_hash,
        file_mime_type,
        duration
    );

    let destination_path = library.file_path(&file_hash);

    if PathBuf::from(local_full_path) != destination_path {
        std::fs::copy(local_full_path, destination_path).map_err(|e| {
            rspc::Error::new(
                rspc::ErrorCode::InternalServerError,
                format!("failed to copy file: {}", e),
            )
        })?;
    }

    let (asset_object_data, file_path_data, asset_object_existed) = library
        .prisma_client()
        ._transaction()
        .run(|client| async move {
            let mut asset_object_existed = false;
            let asset_object_data = match client
                .asset_object()
                .find_unique(asset_object::hash::equals(file_hash.clone()))
                .exec()
                .await?
            {
                Some(asset_object_data) => {
                    asset_object_existed = true;
                    asset_object_data
                }
                None => {
                    client
                        .asset_object()
                        .create(
                            file_hash.clone(),
                            file_size_in_bytes,
                            vec![asset_object::mime_type::set(file_mime_type)],
                        )
                        .exec()
                        .await?
                }
            };
            let (name_wo_ext, ext) = match name.rsplit_once('.') {
                Some((wo_ext, ext)) => (wo_ext, ext),
                None => (name, ""),
            };
            let mut where_clause = vec![
                file_path::materialized_path::equals(materialized_path.into()),
                file_path::name::starts_with(name_wo_ext.into()),
            ];
            if ext != "" {
                where_clause.push(file_path::name::ends_with(format!(".{}", ext)));
            }
            let matches = client.file_path().find_many(where_clause).exec().await?;
            // find the max number "n" in format "name_wo_ext n.ext"
            let max_num = matches
                .iter()
                .filter_map(|file_path_data| {
                    let name = file_path_data.name.as_str();
                    let (name_wo_ext_1, ext_1) = match name.rsplit_once('.') {
                        Some((wo_ext, ext)) => (wo_ext, ext),
                        None => (name, "")
                    };
                    if ext_1 == ext && name_wo_ext_1 == name_wo_ext {
                        return Some(0);
                    }
                    let (name_wo_ext_1, num) = match name_wo_ext_1.rsplit_once(' ') {
                        Some((prefix, num)) => (prefix, num),
                        None => (name_wo_ext_1, "0"),
                    };
                    if ext_1 == ext && name_wo_ext_1 == name_wo_ext {
                        num.parse::<u32>().ok()  // Converts from Result<T, E> to Option<T>
                    } else {
                        None
                    }
                })
                .max();
            let ext_with_dot = if ext == "" { "".to_string() } else { format!(".{}", ext) };
            let new_name = match max_num {
                Some(max_num) => format!("{} {}{}", name_wo_ext, max_num + 1, ext_with_dot),
                None => format!("{}{}", name_wo_ext, ext_with_dot), // same as name
            };
            let file_path_data = client
                .file_path()
                .create(
                    false,
                    materialized_path.to_string(),
                    new_name.clone(),
                    vec![file_path::asset_object_id::set(Some(asset_object_data.id))],
                )
                .exec()
                .await?;
            Ok((asset_object_data, file_path_data, asset_object_existed))
        })
        .await
        .map_err(|e: QueryError| {
            rspc::Error::new(
                rspc::ErrorCode::InternalServerError,
                format!("failed to create asset_object: {}", e),
            )
        })?;

    Ok((file_path_data, asset_object_data, asset_object_existed))
}

// let mut new_name = name.to_string();
// let file_path_data = loop {
//     let res = client
//         .file_path()
//         .create(
//             false,
//             materialized_path.to_string(),
//             new_name.clone(),
//             vec![file_path::asset_object_id::set(Some(asset_object_data.id))],
//         )
//         .exec()
//         .await;
//     if let Err(e) = res {
//         if e.to_string().contains("Unique constraint failed") {
//             tracing::info!("failed to create file_path: {}, retry with a new name", e);
//             let suffix = uuid::Uuid::new_v4()
//                 .to_string()
//                 .split("-")
//                 .next()
//                 .unwrap()
//                 .to_string();
//             new_name = format!("{} ({})", name, suffix);
//             continue;
//         } else {
//             tracing::error!("failed to create file_path: {}", e);
//             return Err(e);
//         }
//     } else {
//         break res.unwrap();
//     }
// };
