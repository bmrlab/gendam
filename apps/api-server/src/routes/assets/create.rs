use prisma_lib::{asset_object, file_path};
use prisma_client_rust::QueryError;
use content_library::Library;
use super::utils::{normalized_materialized_path, contains_invalid_chars, generate_file_hash};

pub async fn create_file_path(
    library: &Library,
    path: &str,
    name: &str,
) -> Result<file_path::Data, rspc::Error> {
    /*
    * TODO
    * 如果 path 是 /a/b/c/, 要确保存在一条数据 {path:"/a/b/",name:"c"}, 不然就是文件夹不存在
    */
    let name = match contains_invalid_chars(name) {
        true => {
            return Err(rspc::Error::new(
                rspc::ErrorCode::BadRequest,
                String::from("name contains invalid chars"),
            ));
        }
        false => name.to_string(),
    };
    let materialized_path = normalized_materialized_path(path);
    let res = library.prisma_client()
        .file_path()
        .create(true, materialized_path, name, vec![])
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
    path: &str,
    local_full_path: &str,
) -> Result<(file_path::Data, asset_object::Data), rspc::Error> {
    let materialized_path = normalized_materialized_path(path);
    // copy file and rename to asset object id
    let file_name = local_full_path.split("/").last().unwrap().to_owned();

    let start_time = std::time::Instant::now();
    // let bytes = std::fs::read(&local_full_path).unwrap();
    // let file_sha256 = sha256::digest(&bytes);
    let fs_metadata = std::fs::metadata(&local_full_path)
        .map_err(|e| rspc::Error::new(
            rspc::ErrorCode::InternalServerError,
            format!("failed to get video metadata: {}", e)
        ))?;
    let file_hash = generate_file_hash(&local_full_path, fs_metadata.len() as u64)
        .await.map_err(|e| rspc::Error::new(
            rspc::ErrorCode::InternalServerError,
            format!("failed to generate file hash: {}", e),
        ))?;
    let duration = start_time.elapsed();
    tracing::info!("{:?}, hash: {:?}, duration: {:?}", local_full_path, file_hash, duration);

    let destination_path = library
        .files_dir
        .join(file_hash.clone());
    std::fs::copy(local_full_path, destination_path).map_err(|e| {
        rspc::Error::new(
            rspc::ErrorCode::InternalServerError,
            format!("failed to copy file: {}", e),
        )
    })?;

    let (asset_object_data, file_path_data) = library.prisma_client()
        ._transaction()
        .run(|client| async move {
            let asset_object_data = client
                .asset_object()
                .upsert(
                    asset_object::hash::equals(file_hash.clone()),
                    asset_object::create(file_hash.clone(), vec![]),
                    vec![],
                )
                .exec()
                .await
                .map_err(|e| {
                    tracing::error!("failed to create asset_object: {}", e);
                    e
                })?;
            let mut new_file_name = file_name.clone();
            let file_path_data = loop {
                let res = client
                    .file_path()
                    .create(
                        false,
                        materialized_path.clone(),
                        new_file_name.clone(),
                        vec![file_path::asset_object_id::set(Some(asset_object_data.id))],
                    )
                    .exec()
                    .await;
                if let Err(e) = res {
                    if e.to_string().contains("Unique constraint failed") {
                        tracing::info!("failed to create file_path: {}, retry with a new name", e);
                        let suffix = uuid::Uuid::new_v4().to_string().split("-").next().unwrap().to_string();
                        new_file_name = format!("{} ({})", file_name, suffix);
                        continue;
                    } else {
                        tracing::error!("failed to create file_path: {}", e);
                        return Err(e);
                    }
                } else {
                    break res.unwrap();
                }
            };
            Ok((asset_object_data, file_path_data))
        })
        .await
        .map_err(|e: QueryError| {
            rspc::Error::new(
                rspc::ErrorCode::InternalServerError,
                format!("failed to create asset_object: {}", e),
            )
        })?;

    Ok((file_path_data, asset_object_data))
}
