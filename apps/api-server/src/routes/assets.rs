use serde::{
    Serialize,
    Deserialize
};
use specta::Type;
use prisma_lib::{
    // asset_object,
    file_path,
    new_client_with_url
};
use serde_json::json;
use rspc::Router;
use crate::{Ctx, R};

#[derive(Deserialize, Type, Debug)]
struct FilePathCreatePayload {
    path: String,
    name: String,
}

#[derive(Deserialize, Type, Debug)]
struct FilePathQueryPayload {
    path: String,
    #[serde(rename = "dirsOnly")]
    dirs_only: bool,
}

#[derive(Serialize, Type, Debug)]
struct FilePathQueryResult {
    id: i32,
    name: String,
    #[serde(rename = "isDir")]
    is_dir: bool,
}

// fn server_error() {
//     Err(rspc::Error::new(
//         rspc::ErrorCode,
//         String::from("path muse be start with /")
//     ))
// }

pub fn get_routes() -> Router<Ctx> {
    let router = R.router()
    .procedure("create_file_path",
        R.mutation(|ctx, input: FilePathCreatePayload| async move {
            let client = new_client_with_url(ctx.library.db_url.as_str())
                .await
                .map_err(|e| rspc::Error::new(
                    rspc::ErrorCode::InternalServerError,
                    format!("failed to create prisma client: {}", e)
                ))?;
            /*
             * TODO
             * 如果 path 是 /a/b/c/, 要确保存在一条数据 {path:"/a/b/",name:"c"}, 不然就是文件夹不存在
             */
            let materialized_path = if input.path.ends_with("/") {
                input.path
            } else {
                format!("{}/", input.path)
            };
            let res = client
                .file_path()
                .create(
                    true,
                    materialized_path,
                    input.name,
                    vec![],
                )
                .exec().await
                .map_err(|e| rspc::Error::new(
                    rspc::ErrorCode::InternalServerError,
                    format!("failed to create file_path: {}", e)
                ))?;
            Ok(json!(res).to_string())
        })
    ).procedure("create_asset_object",
        R.mutation(|ctx, input: FilePathCreatePayload| async move {
            let client = new_client_with_url(ctx.library.db_url.as_str())
                .await
                .map_err(|e| rspc::Error::new(
                    rspc::ErrorCode::InternalServerError,
                    format!("failed to create prisma client: {}", e)
                ))?;
            let new_asset_object_record = client
                .asset_object()
                .create(vec![])
                .exec().await
                .map_err(|e| rspc::Error::new(
                    rspc::ErrorCode::InternalServerError,
                    format!("failed to create asset_object: {}", e)
                ))?;
            let materialized_path = if input.path.ends_with("/") {
                input.path
            } else {
                format!("{}/", input.path)
            };
            let res = client
                .file_path()
                .create(
                    false,
                    materialized_path,
                    input.name,
                    vec![
                        // file_path::SetParam::SetId(new_asset_object_record.id)
                        file_path::assset_object_id::set(Some(new_asset_object_record.id))
                    ],
                )
                .exec().await
                .map_err(|e| rspc::Error::new(
                    rspc::ErrorCode::InternalServerError,
                    format!("failed to create file_path: {}", e)
                ))?;
            Ok(json!(res).to_string())
        })
    )
    .procedure("list",
        R.query(|ctx, input: FilePathQueryPayload| async move {
            let client = new_client_with_url(ctx.library.db_url.as_str())
                .await
                .map_err(|e| rspc::Error::new(
                    rspc::ErrorCode::InternalServerError,
                    format!("failed to create prisma client: {}", e)
                ))?;
            let mut where_params = vec![
                file_path::materialized_path::equals(input.path),
            ];
            if input.dirs_only {
                where_params.push(file_path::is_dir::equals(true));
            }
            let res = client
                .file_path()
                .find_many(where_params)
                .exec().await
                .map_err(|e| rspc::Error::new(
                    rspc::ErrorCode::InternalServerError,
                    format!("failed to list dirs: {}", e)
                ))?;
            let names: Vec<FilePathQueryResult> = res.iter().map(|r| {
                FilePathQueryResult {
                    id: r.id,
                    name: r.name.clone(),
                    is_dir: r.is_dir,
                }
            }).collect::<Vec::<_>>();
            // Ok(json!(names).to_string())
            Ok(names)
        })
    );
    router
}
