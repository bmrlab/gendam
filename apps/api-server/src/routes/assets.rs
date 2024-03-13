use prisma_lib::{asset_object, file_path};
use rspc::{Router, Rspc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use specta::Type;
// use crate::{Ctx, R};
use crate::task_queue::create_video_task;
use crate::CtxWithLibrary;

use content_library::Library;

#[derive(Deserialize, Type, Debug)]
struct FilePathCreatePayload {
    path: String,
    name: String,
}

#[derive(Deserialize, Type, Debug)]
#[serde(rename_all = "camelCase")]
struct AssetObjectCreatePayload {
    path: String,
    local_full_path: String,
}

#[derive(Deserialize, Type, Debug)]
struct FilePathQueryPayload {
    path: String,
    #[serde(rename = "dirsOnly")]
    dirs_only: bool,
}

#[derive(Serialize, Type, Debug)]
#[serde(rename_all = "camelCase")]
struct AssetObjectQueryResult {
    id: i32,
    // note: String,
    local_full_path: String,
}

#[derive(Serialize, Type, Debug)]
#[serde(rename_all = "camelCase")]
struct FilePathQueryResult {
    id: i32,
    name: String,
    is_dir: bool,
    asset_object: Option<AssetObjectQueryResult>,
}

// fn server_error() {
//     Err(rspc::Error::new(
//         rspc::ErrorCode,
//         String::from("path muse be start with /")
//     ))
// }

pub fn get_routes<TCtx>() -> Router<TCtx>
where
    TCtx: CtxWithLibrary + Clone + Send + Sync + 'static,
{
    let router = Rspc::<TCtx>::new()
        .router()
        .procedure(
            "create_file_path",
            Rspc::<TCtx>::new().mutation(|ctx, input: FilePathCreatePayload| async move {
                let library = ctx.library()?;
                /*
                 * TODO
                 * 如果 path 是 /a/b/c/, 要确保存在一条数据 {path:"/a/b/",name:"c"}, 不然就是文件夹不存在
                 */
                let materialized_path = if input.path.ends_with("/") {
                    input.path
                } else {
                    format!("{}/", input.path)
                };
                let res = library.prisma_client()
                    .file_path()
                    .create(true, materialized_path, input.name, vec![])
                    .exec()
                    .await
                    .map_err(|e| {
                        rspc::Error::new(
                            rspc::ErrorCode::InternalServerError,
                            format!("failed to create file_path: {}", e),
                        )
                    })?;
                Ok(json!(res).to_string())
            }),
        )
        .procedure(
            "create_asset_object",
            Rspc::<TCtx>::new().mutation(|ctx, input: AssetObjectCreatePayload| async move {
                let library = ctx.library()?;
                let (file_path_data, asset_object_data) = create_asset_object(&library, &input.path, &input.local_full_path).await?;

                let tx = ctx.get_task_tx();
                create_video_task(
                    &file_path_data.materialized_path,
                    &asset_object_data, &ctx, tx,
                )
                    .await
                    .map_err(|_| {
                        rspc::Error::new(
                            rspc::ErrorCode::InternalServerError,
                            format!("failed to create video task"),
                        )
                    })?;

                Ok(json!(file_path_data).to_string())
            }),
        )
        .procedure(
            "list",
            Rspc::<TCtx>::new().query(|ctx, input: FilePathQueryPayload| async move {
                let library = ctx.library()?;
                let mut where_params = vec![file_path::materialized_path::equals(input.path)];
                if input.dirs_only {
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
                let names: Vec<FilePathQueryResult> = res
                    .iter()
                    .map(|r| FilePathQueryResult {
                        id: r.id,
                        name: r.name.clone(),
                        is_dir: r.is_dir,
                        asset_object: match r.asset_object.as_ref() {
                            Some(asset_object) => match asset_object {
                                None => None,
                                Some(asset_object) => Some(AssetObjectQueryResult {
                                    id: asset_object.id,
                                    local_full_path: format!(
                                        "{}/{}",
                                        library.files_dir.to_str().unwrap(),
                                        asset_object.id
                                    ),
                                }),
                            },
                            None => None,
                        },
                    })
                    .collect::<Vec<_>>();
                // Ok(json!(names).to_string())
                Ok(names)
            }),
        )
        .procedure(
            "process_video_asset",
            Rspc::<TCtx>::new().mutation(|ctx, file_path_id: i32| async move {
                let library = ctx.library()?;
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
                    &asset_object_data, &ctx, tx
                ).await {
                    Ok(res) => Ok(serde_json::to_value(res).unwrap()),
                    Err(_) => Err(rspc::Error::new(
                        rspc::ErrorCode::InternalServerError,
                        String::from("failed to create video task"),
                    )),
                }
            }),
        );
    router
}


async fn create_asset_object(
    library: &Library, path: &str, local_full_path: &str
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
                file_path::assset_object_id::set(Some(asset_object_data.id)),
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
