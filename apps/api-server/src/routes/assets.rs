use prisma_lib::{asset_object, file_path};
use prisma_client_rust::{PrismaValue, raw};
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

fn contains_invalid_chars(name: &str) -> bool {
    name.chars().any(|c| match c {
        '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => true,
        _ => false,
    })
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
            }),
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
            }),
        )
        .procedure(
            "rename_file_path",
            Rspc::<TCtx>::new().mutation({
                #[derive(Deserialize, Type, Debug)]
                struct FilePathRenamePayload {
                    path: String,
                    old_name: String,
                    new_name: String,
                }
                |ctx, input: FilePathRenamePayload| async move {
                    let library = ctx.library()?;
                    rename_file_path(&library, &input.path, &input.old_name, &input.new_name)
                        .await?;
                    Ok(())
                }
            }),
        )
        .procedure(
            "process_video_asset",
            Rspc::<TCtx>::new().mutation(|ctx, input: i32| async move {
                let library = ctx.library()?;
                let file_path_id = input;
                process_video_asset(&library, &ctx, file_path_id).await?;
                Ok(())
            }),
        );
    router
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
    let materialized_path = if path.ends_with("/") {
        path.to_string()
    } else {
        format!("{}/", path)
    };
    let name = match contains_invalid_chars(name) {
        true => {
            return Err(rspc::Error::new(
                rspc::ErrorCode::BadRequest,
                String::from("name contains invalid chars"),
            ));
        }
        false => name.to_string(),
    };
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
    // create asset object record
    let asset_object_data = library.prisma_client()
        .asset_object()
        .create(vec![])
        .exec()
        .await
        .map_err(|e| {
            rspc::Error::new(
                rspc::ErrorCode::InternalServerError,
                format!("failed to create asset_object: {}", e),
            )
        })?;

    // copy file and rename to asset object id
    let materialized_path = if path.ends_with("/") {
        path.to_owned()
    } else {
        format!("{}/", path)
    };
    let file_name = local_full_path.split("/").last().unwrap().to_owned();
    let destination_path = library
        .files_dir
        .join(asset_object_data.id.to_string());
    std::fs::copy(local_full_path, destination_path).map_err(|e| {
        rspc::Error::new(
            rspc::ErrorCode::InternalServerError,
            format!("failed to copy file: {}", e),
        )
    })?;

    // create file_path
    let file_path_data = library.prisma_client()
        .file_path()
        .create(
            false,
            materialized_path,
            file_name,
            vec![
                // file_path::SetParam::SetId(asset_object_data.id)
                file_path::asset_object_id::set(Some(asset_object_data.id)),
            ],
        )
        .exec()
        .await
        .map_err(|e| {
            rspc::Error::new(
                rspc::ErrorCode::InternalServerError,
                format!("failed to create file_path: {}", e),
            )
        })?;

    Ok((file_path_data, asset_object_data))
}


async fn rename_file_path(
    library: &Library,
    path: &str,
    old_name: &str,
    new_name: &str,
) -> Result<(), rspc::Error> {
    let materialized_path = if path.ends_with("/") {
        path.to_string()
    } else {
        format!("{}/", path)
    };
    let old_name = old_name.to_string();
    let new_name = match contains_invalid_chars(new_name) {
        true => {
            return Err(rspc::Error::new(
                rspc::ErrorCode::BadRequest,
                String::from("name contains invalid chars"),
            ));
        }
        false => new_name.to_string(),
    };
    let old_materialized_path = format!("{}{}/", &materialized_path, &old_name);
    let new_materialized_path = format!("{}{}/", &materialized_path, &new_name);

    library.prisma_client()
        .file_path()
        .update(
            file_path::materialized_path_name(materialized_path, old_name),
            vec![file_path::name::set(new_name)],
        )
        .exec()
        .await
        .map_err(|e| {
            rspc::Error::new(
                rspc::ErrorCode::InternalServerError,
                format!("failed to rename file_path item: {}", e),
            )
        })?;

    /*
     * TODO: 要区分一下是文件夹重命名还是文件重命名，如果是文件，下面的不需要
     * https://github.com/bmrlab/tauri-dam-test-playground/issues/15#issuecomment-2001923972
     */

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
}

#[derive(Serialize, Type, Debug)]
#[serde(rename_all = "camelCase")]
struct FilePathQueryResult {
    id: i32,
    name: String,
    is_dir: bool,
    asset_object: Option<AssetObjectQueryResult>,
}

async fn list_file_path(
    library: &Library,
    path: &str,
    dirs_only: bool,
) -> Result<Vec<FilePathQueryResult>, rspc::Error> {
    let mut where_params = vec![file_path::materialized_path::equals(path.to_string())];
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
            is_dir: r.is_dir,
            asset_object: match r.asset_object.as_ref() {
                Some(asset_object) => match asset_object {
                    None => None,
                    Some(asset_object) => Some(AssetObjectQueryResult {
                        id: asset_object.id
                    }),
                },
                None => None,
            },
        })
        .collect::<Vec<FilePathQueryResult>>();

    Ok(res)
}
