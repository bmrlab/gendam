use prisma_lib::{asset_object, file_path};
use prisma_client_rust::{PrismaValue, raw, QueryError};
use rspc::{Router, Rspc};
use serde::{Deserialize, Serialize};
// use serde_json::json;
use specta::Type;
// use crate::{Ctx, R};
use crate::task_queue::create_video_task;
use crate::CtxWithLibrary;

use content_library::Library;

// fn server_error() {
//     Err(rspc::Error::new(
//         rspc::ErrorCode,
//         String::from("path muse be start with /")
//     ))
// }

#[derive(Deserialize, Type, Debug)]
#[serde(rename_all = "camelCase")]
struct FilePathRequestPayload {
    id: i32,
    is_dir: bool,
    path: String,
    name: String,
}

pub fn get_routes<TCtx>() -> Router<TCtx>
where
    TCtx: CtxWithLibrary + Clone + Send + Sync + 'static,
{
    let router = Rspc::<TCtx>::new()
        .router()
        .procedure(
            "create_file_path",
            Rspc::<TCtx>::new().mutation({
                #[derive(Deserialize, Type, Debug)]
                struct FilePathCreatePayload {
                    path: String,
                    name: String,
                }
                |ctx, input: FilePathCreatePayload| async move {
                    let library = ctx.library()?;
                    create_file_path(&library, &input.path, &input.name).await?;
                    Ok(())
                }
            }),
        )
        .procedure(
            "create_asset_object",
            Rspc::<TCtx>::new().mutation({
                #[derive(Deserialize, Type, Debug)]
                #[serde(rename_all = "camelCase")]
                struct AssetObjectCreatePayload {
                    path: String,
                    local_full_path: String,
                }
                |ctx: TCtx, input: AssetObjectCreatePayload| async move {
                    let library = ctx.library()?;
                    let (file_path_data, _asset_object_data) =
                        create_asset_object(&library, &input.path, &input.local_full_path)
                        .await?;
                    process_video_asset(&library, &ctx, file_path_data.id).await?;
                    // Ok(json!(file_path_data).to_string())
                    Ok(())
                }
            })
        )
        .procedure(
            "list",
            Rspc::<TCtx>::new().query({
                #[derive(Deserialize, Type, Debug)]
                struct FilePathQueryPayload {
                    path: String,
                    #[serde(rename = "dirsOnly")]
                    dirs_only: bool,
                }
                |ctx, input: FilePathQueryPayload| async move {
                    let library = ctx.library()?;
                    let names =
                        list_file_path(&library, &input.path, input.dirs_only)
                        .await?;
                    Ok(names)
                }
            })
        )
        .procedure(
            "rename_file_path",
            Rspc::<TCtx>::new().mutation({
                #[derive(Deserialize, Type, Debug)]
                #[serde(rename_all = "camelCase")]
                struct FilePathRenamePayload {
                    id: i32,
                    is_dir: bool,
                    path: String,
                    old_name: String,
                    new_name: String,
                }
                |ctx, input: FilePathRenamePayload| async move {
                    let library = ctx.library()?;
                    rename_file_path(
                        &library, input.id, input.is_dir,
                        &input.path, &input.old_name, &input.new_name
                    ).await?;
                    Ok(())
                }
            })
        )
        .procedure(
            "move_file_path",
            Rspc::<TCtx>::new().mutation({
                #[derive(Deserialize, Type, Debug)]
                #[serde(rename_all = "camelCase")]
                struct FilePathMovePayload {
                    active: FilePathRequestPayload,
                    target: FilePathRequestPayload,
                }
                |ctx, input: FilePathMovePayload| async move {
                    let library = ctx.library()?;
                    move_file_path(
                        &library, input.active, input.target
                    ).await?;
                    Ok(())
                }
            })
        )
        .procedure(
            "delete_file_path",
            Rspc::<TCtx>::new().mutation({
                #[derive(Deserialize, Type, Debug)]
                #[serde(rename_all = "camelCase")]
                struct FilePathDeletePayload {
                    path: String,
                    name: String,
                }
                |ctx, input: FilePathDeletePayload| async move {
                    let library = ctx.library()?;
                    delete_file_path(&library, &input.path, &input.name).await?;
                    Ok(())
                }
            })
        )
        .procedure(
            "process_video_asset",
            Rspc::<TCtx>::new().mutation(|ctx, input: i32| async move {
                let library = ctx.library()?;
                let file_path_id = input;
                process_video_asset(&library, &ctx, file_path_id).await?;
                Ok(())
            })
        );
    router
}

fn contains_invalid_chars(name: &str) -> bool {
    name.chars().any(|c| match c {
        '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => true,
        _ => false,
    })
}

fn normalized_materialized_path(path: &str) -> String {
    if path.ends_with("/") {
        path.to_string()
    } else {
        format!("{}/", path)
    }
}

async fn create_file_path(
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


async fn create_asset_object(
    library: &Library,
    path: &str,
    local_full_path: &str,
) -> Result<(file_path::Data, asset_object::Data), rspc::Error> {
    let materialized_path = normalized_materialized_path(path);
    // copy file and rename to asset object id
    let file_name = local_full_path.split("/").last().unwrap().to_owned();

    let start_time = std::time::Instant::now();
    let bytes = std::fs::read(&local_full_path).unwrap();
    let file_sha256 = sha256::digest(&bytes);
    let duration = start_time.elapsed();
    tracing::info!("{:?}, sha256: {:?}, duration: {:?}", local_full_path, file_sha256, duration);

    let destination_path = library
        .files_dir
        .join(file_sha256.clone());
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
                    asset_object::hash::equals(file_sha256.clone()),
                    asset_object::create(file_sha256.clone(), vec![]),
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


async fn rename_file_path(
    library: &Library,
    id: i32,
    is_dir: bool,
    path: &str,
    old_name: &str,
    new_name: &str,
) -> Result<(), rspc::Error> {
    // TODO: 所有 SQL 要放进一个 transaction 里面

    if contains_invalid_chars(new_name) {
        return Err(rspc::Error::new(
            rspc::ErrorCode::BadRequest,
            String::from("name contains invalid chars"),
        ));
    }
    let materialized_path = normalized_materialized_path(path);
    let file_path_data = library.prisma_client()
        .file_path()
        .find_first(vec![
            file_path::id::equals(id),
            file_path::materialized_path::equals(materialized_path.clone()),
            file_path::is_dir::equals(is_dir),
            file_path::name::equals(old_name.to_string()),
        ])
        .exec().await.map_err(|e| {
            rspc::Error::new(
                rspc::ErrorCode::InternalServerError,
                format!("failed to find file_path: {}", e),
            )
        })?;
    if let None = file_path_data {
        return Err(rspc::Error::new(
            rspc::ErrorCode::NotFound,
            String::from("file_path not found"),
        ));
    }

    library.prisma_client()
        .file_path()
        .update(
            file_path::materialized_path_name(materialized_path.clone(), old_name.to_string()),
            vec![file_path::name::set(new_name.to_string())],
        )
        .exec()
        .await
        .map_err(|e| {
            rspc::Error::new(
                rspc::ErrorCode::InternalServerError,
                format!("failed to rename file_path item: {}", e),
            )
        })?;

    // 要区分一下是文件夹重命名还是文件重命名，如果是文件，下面的不需要
    if !is_dir {
        return Ok(());
    }

    /*
     * https://github.com/bmrlab/tauri-dam-test-playground/issues/15#issuecomment-2001923972
     */
    let old_materialized_path = format!("{}{}/", &materialized_path, &old_name);
    let new_materialized_path = format!("{}{}/", &materialized_path, &new_name);
    let old_materialized_path_like = format!("{}%", &old_materialized_path);
    library.prisma_client()
        ._execute_raw(raw!(
            r#"
            UPDATE FilePath
            SET materializedPath = $1 || SUBSTR(materializedPath, LENGTH($2) + 1)
            WHERE materializedPath LIKE $3
            "#,
            // 注意，这里的顺序一定要 $1,$2,$3, 序号似乎没有被遵守
            PrismaValue::String(new_materialized_path),
            PrismaValue::String(old_materialized_path),
            PrismaValue::String(old_materialized_path_like)
        ))
        // .update_many(
        //     vec![file_path::materialized_path::starts_with(old_materialized_path)],
        //     vec![file_path::materialized_path::set(new_materialized_path)],
        // )
        .exec()
        .await
        .map_err(|e| {
            rspc::Error::new(
                rspc::ErrorCode::InternalServerError,
                format!("failed to rename file_path for children: {}", e),
            )
        })?;

    Ok(())
}


async fn move_file_path(
    library: &Library,
    mut active: FilePathRequestPayload,
    mut target: FilePathRequestPayload,
) -> Result<(), rspc::Error> {
    // TODO: 所有 SQL 要放进一个 transaction 里面

    // 其实不应该对 path 做 normalize，调用接口的时候要确保格式正确
    active.path = normalized_materialized_path(&active.path);
    target.path = normalized_materialized_path(&target.path);

    let sql_error = |e: QueryError| {
        rspc::Error::new(
            rspc::ErrorCode::InternalServerError,
            format!("sql query failed: {}", e),
        )
    };

    let active_file_path_data = library.prisma_client()
        .file_path()
        .find_first(vec![
            file_path::id::equals(active.id),
            file_path::materialized_path::equals(active.path.clone()),
            file_path::is_dir::equals(active.is_dir),
            file_path::name::equals(active.name.clone()),
        ])
        .exec().await.map_err(sql_error)?;
    let active_file_path_data = match active_file_path_data {
        Some(t) => t,
        None => {
            return Err(rspc::Error::new(
                rspc::ErrorCode::NotFound,
                String::from("active file_path not found"),
            ));
        }
    };

    // TODO: 首先，确保 target.is_dir == true
    let target_file_path_data = library.prisma_client()
        .file_path()
        .find_first(vec![
            file_path::id::equals(target.id),
            file_path::materialized_path::equals(target.path.clone()),
            file_path::is_dir::equals(true),
            file_path::name::equals(target.name.clone()),
        ])
        .exec().await.map_err(sql_error)?;
    let _target_file_path_data = match target_file_path_data {
        Some(t) => t,
        None => {
            return Err(rspc::Error::new(
                rspc::ErrorCode::NotFound,
                String::from("target file_path not found"),
            ));
        }
    };

    // 确保 target 下不存在相同名字的文件，不然移动失败
    let duplicated_file_path_data = library.prisma_client()
        .file_path()
        .find_first(vec![
            file_path::id::equals(target.id),
            file_path::materialized_path::equals(target.path.clone()),
            file_path::name::equals(active.name.clone()),
        ])
        .exec().await.map_err(sql_error)?;
    if let Some(data) = duplicated_file_path_data {
        return Err(rspc::Error::new(
            rspc::ErrorCode::BadRequest,
            format!("file_path already exists: {:?}", data),
        ));
    }

    // rename file_path
    let new_materialized_path = format!("{}{}/", target.path.as_str(), target.name.as_str());
    library.prisma_client()
        .file_path()
        .update(
            file_path::id::equals(active_file_path_data.id),
            vec![file_path::materialized_path::set(new_materialized_path)],
        )
        .exec().await.map_err(sql_error)?;

    if !active.is_dir {
        return Ok(());
    }

    /*
     * rename children items
     * /a/aa/x
     * /a/aa/x/y1
     * /a/aa/x/y2
     *
     * /a/aa/x -> /a/bb/cc/x
     * /a/aa/x/y1 -> /a/bb/cc/x/y1
     * /a/aa/x/y2 -> /a/bb/cc/x/y2
     *
     * Same as rename
     */
    let new_materialized_path = format!("{}{}/{}/", target.path.as_str(), target.name.as_str(), active.name.as_str());
    let old_materialized_path = format!("{}{}/", active.path.as_str(), active.name.as_str());
    let old_materialized_path_like = format!("{}%", &old_materialized_path);
    library.prisma_client()
        ._execute_raw(raw!(
            r#"
            UPDATE FilePath
            SET materializedPath = $1 || SUBSTR(materializedPath, LENGTH($2) + 1)
            WHERE materializedPath LIKE $3
            "#,
            // 注意，这里的顺序一定要 $1,$2,$3, 序号似乎没有被遵守
            PrismaValue::String(new_materialized_path),
            PrismaValue::String(old_materialized_path),
            PrismaValue::String(old_materialized_path_like)
        ))
        .exec()
        .await
        .map_err(|e| {
            rspc::Error::new(
                rspc::ErrorCode::InternalServerError,
                format!("failed to move file_path children: {}", e),
            )
        })?;

    Ok(())
}


/**
 * TODO: 删除 file_path 以后要检查一下 assetobject 是否还有其他引用，如果没有的话，要删除 assetobject，进一步的，删除 filehandlertask
 */
async fn delete_file_path(
    library: &Library,
    path: &str,
    name: &str,
) -> Result<(), rspc::Error> {
    let materialized_path = normalized_materialized_path(path);
    let name = name.to_string();

    library.prisma_client()
        ._transaction()
        .run(|client| async move {
            client
                .file_path()
                .delete(file_path::materialized_path_name(materialized_path.clone(), name.clone()))
                .exec()
                .await
                .map_err(|e| {
                    tracing::error!("failed to delete file_path item: {}", e);
                    e
                })?;
            let materialized_path_startswith = format!("{}{}/", &materialized_path, &name);
            client
                .file_path()
                .delete_many(vec![
                    file_path::materialized_path::starts_with(materialized_path_startswith)
                ])
                .exec()
                .await
                .map_err(|e| {
                    tracing::error!("failed to delete file_path for children: {}", e);
                    e
                })?;
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


async fn process_video_asset(
    library: &Library,
    ctx: &impl CtxWithLibrary,
    file_path_id: i32,
) -> Result<(), rspc::Error> {
    let tx = ctx.get_task_tx();
    let file_path_data = library.prisma_client()
        .file_path()
        .find_unique(file_path::id::equals(file_path_id))
        .with(file_path::asset_object::fetch())
        .exec()
        .await
        .map_err(|e| {
            rspc::Error::new(
                rspc::ErrorCode::InternalServerError,
                format!("failed to find file_path: {}", e),
            )
        })?
        .ok_or_else(|| {
            rspc::Error::new(
                rspc::ErrorCode::InternalServerError,
                format!("failed to find file_path"),
            )
        })?;

    let asset_object_data = file_path_data.asset_object
        .unwrap()
        .ok_or_else(|| {
            rspc::Error::new(
                rspc::ErrorCode::InternalServerError,
                String::from("file_path.asset_object is None"),
            )
        })?;
    // let asset_object_data = *asset_object_data;

    match create_video_task(
        &file_path_data.materialized_path,
        &asset_object_data,
        ctx,
        tx
    ).await {
        Ok(_) => Ok(()),
        Err(_) => Err(rspc::Error::new(
            rspc::ErrorCode::InternalServerError,
            String::from("failed to create video task"),
        )),
    }
}


#[derive(Serialize, Type, Debug)]
#[serde(rename_all = "camelCase")]
struct AssetObjectQueryResult {
    id: i32,
    hash: String,
}

#[derive(Serialize, Type, Debug)]
#[serde(rename_all = "camelCase")]
struct FilePathQueryResult {
    id: i32,
    name: String,
    materialized_path: String,
    is_dir: bool,
    asset_object: Option<AssetObjectQueryResult>,
}

async fn list_file_path(
    library: &Library,
    path: &str,
    dirs_only: bool,
) -> Result<Vec<FilePathQueryResult>, rspc::Error> {
    let materialized_path = normalized_materialized_path(path);
    let mut where_params = vec![file_path::materialized_path::equals(materialized_path)];
    if dirs_only {
        where_params.push(file_path::is_dir::equals(true));
    }
    let res = library.prisma_client()
        .file_path()
        .find_many(where_params)
        .with(file_path::asset_object::fetch())
        .exec()
        .await
        .map_err(|e| {
            rspc::Error::new(
                rspc::ErrorCode::InternalServerError,
                format!("failed to list dirs: {}", e),
            )
        })?;
    // TODO 这里写的有点挫
    let res = res
        .iter()
        .map(|r| FilePathQueryResult {
            id: r.id,
            name: r.name.clone(),
            materialized_path: r.materialized_path.clone(),
            is_dir: r.is_dir,
            asset_object: match r.asset_object.as_ref() {
                Some(asset_object) => match asset_object {
                    None => None,
                    Some(asset_object) => Some(AssetObjectQueryResult {
                        id: asset_object.id,
                        hash: asset_object.hash.clone(),
                    }),
                },
                None => None,
            },
        })
        .collect::<Vec<FilePathQueryResult>>();

    Ok(res)
}
