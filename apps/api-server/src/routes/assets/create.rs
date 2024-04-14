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
    Ok(res)
}

pub async fn create_asset_object(
    library: &Library,
    materialized_path: &str,
    name: &str,
    local_full_path: &str,
) -> Result<(file_path::Data, asset_object::Data, bool), rspc::Error> {
    let start_time = std::time::Instant::now();
    // let bytes = std::fs::read(&local_full_path).unwrap();
    // let file_sha256 = sha256::digest(&bytes);
    let fs_metadata = std::fs::metadata(&local_full_path).map_err(|e| {
        rspc::Error::new(
            rspc::ErrorCode::InternalServerError,
            format!("failed to get video metadata: {}", e),
        )
    })?;
    let guess = mime_guess::from_path(&local_full_path);
    let file_mime_type = match guess.first() {
        Some(mime) => Some(mime.to_string()),
        None => None,
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
        "{:?}, hash: {:?}, duration: {:?}",
        local_full_path,
        file_hash,
        duration
    );

    let destination_path = library.file_path(&file_hash);
    std::fs::copy(local_full_path, destination_path).map_err(|e| {
        rspc::Error::new(
            rspc::ErrorCode::InternalServerError,
            format!("failed to copy file: {}", e),
        )
    })?;

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
                            vec![asset_object::mime_type::set(file_mime_type)]
                        )
                        .exec()
                        .await?
                }
            };
            let mut new_name = name.to_string();
            let file_path_data = loop {
                let res = client
                    .file_path()
                    .create(
                        false,
                        materialized_path.to_string(),
                        new_name.clone(),
                        vec![file_path::asset_object_id::set(Some(asset_object_data.id))],
                    )
                    .exec()
                    .await;
                if let Err(e) = res {
                    if e.to_string().contains("Unique constraint failed") {
                        tracing::info!("failed to create file_path: {}, retry with a new name", e);
                        let suffix = uuid::Uuid::new_v4()
                            .to_string()
                            .split("-")
                            .next()
                            .unwrap()
                            .to_string();
                        new_name = format!("{} ({})", name, suffix);
                        continue;
                    } else {
                        tracing::error!("failed to create file_path: {}", e);
                        return Err(e);
                    }
                } else {
                    break res.unwrap();
                }
            };
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
