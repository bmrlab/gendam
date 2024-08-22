pub mod types;

use crate::{routes::assets::process::process_asset, CtxWithLibrary};
use content_base::{
    delete::DeletePayload, task::CancelTaskPayload, ContentMetadata, ContentTaskType,
};
use prisma_client_rust::QueryError;
use prisma_lib::{asset_object, file_handler_task};
use rspc::{Router, RouterBuilder};
use serde::Deserialize;
use specta::Type;

use super::assets::process::generate_thumbnail;

#[derive(Deserialize, Type, Debug)]
#[serde(rename_all = "camelCase")]
struct TaskListRequestFilter {
    #[serde(skip_serializing_if = "Option::is_none")]
    asset_object_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    asset_object_ids: Option<Vec<i32>>,
}

#[derive(Deserialize, Type, Debug)]
#[serde(rename_all = "camelCase")]
struct TaskListRequestPayload {
    filter: TaskListRequestFilter,
}

#[derive(Deserialize, Type, Debug)]
#[serde(rename_all = "camelCase")]
struct TaskCancelRequestPayload {
    asset_object_id: i32,
    task_types: Option<Vec<String>>,
}

#[derive(Deserialize, Type, Debug)]
#[serde(rename_all = "camelCase")]
struct TaskRedoRequestPayload {
    asset_object_id: i32,
}

fn sql_error(e: QueryError) -> rspc::Error {
    rspc::Error::new(
        rspc::ErrorCode::InternalServerError,
        format!("sql query failed: {}", e),
    )
}

fn convert_anyhow_error(e: anyhow::Error) -> rspc::Error {
    rspc::Error::new(
        rspc::ErrorCode::InternalServerError,
        format!("error: {}", e),
    )
}

pub fn get_routes<TCtx>() -> RouterBuilder<TCtx>
where
    TCtx: CtxWithLibrary + Clone + Send + Sync + 'static,
{
    Router::<TCtx>::new()
        .query("list", |t| {
            t(|ctx: TCtx, payload: TaskListRequestPayload| async move {
                let library = ctx.library()?;
                let mut whera_params: Vec<prisma_lib::file_handler_task::WhereParam> = vec![];
                if let Some(asset_object_id) = payload.filter.asset_object_id {
                    whera_params.push(prisma_lib::file_handler_task::asset_object_id::equals(
                        asset_object_id,
                    ));
                }
                if let Some(asset_object_ids) = payload.filter.asset_object_ids {
                    whera_params.push(prisma_lib::file_handler_task::asset_object_id::in_vec(
                        asset_object_ids,
                    ));
                }
                let res = library
                    .prisma_client()
                    .file_handler_task()
                    .find_many(whera_params)
                    .exec()
                    .await
                    .map_err(sql_error)?;
                Ok(res)
            })
        })
        .query("get_assets_in_process", |t| {
            t(|ctx: TCtx, _input: ()| async move {
                let library = ctx.library()?;
                let asset_object_data_list = library
                    .prisma_client()
                    .asset_object()
                    .find_many(vec![prisma_lib::asset_object::tasks::some(vec![
                        prisma_lib::file_handler_task::exit_code::equals(None),
                    ])])
                    .with(
                        prisma_lib::asset_object::file_paths::fetch(vec![])
                            .order_by(prisma_lib::file_path::created_at::order(
                                prisma_client_rust::Direction::Desc,
                            ))
                            .take(1),
                    )
                    .exec()
                    .await
                    .map_err(sql_error)?;
                let file_path_data_list = asset_object_data_list
                    .into_iter()
                    .filter_map(|mut asset_object_data| {
                        let file_paths = asset_object_data.file_paths.take();
                        // leave asset_object_data.file_paths as None
                        match file_paths {
                            Some(file_paths) => {
                                if file_paths.len() > 0 {
                                    let mut file_path_data = file_paths[0].clone();
                                    file_path_data.asset_object =
                                        Some(Some(Box::new(asset_object_data)));
                                    Some(file_path_data)
                                } else {
                                    None
                                }
                            }
                            None => None,
                        }
                    })
                    .collect::<Vec<prisma_lib::file_path::Data>>();
                Ok(file_path_data_list)
            })
        })
        .mutation("cancel", |t| {
            t(|ctx: TCtx, input: TaskCancelRequestPayload| async move {
                let library = ctx.library()?;
                let asset_object = library
                    .prisma_client()
                    .asset_object()
                    .find_first(vec![asset_object::id::equals(input.asset_object_id)])
                    .exec()
                    .await
                    .map_err(sql_error)?
                    .ok_or(anyhow::anyhow!("not found"))
                    .map_err(convert_anyhow_error)?;

                let content_base = ctx.content_base()?;
                let payload = CancelTaskPayload::new(&asset_object.hash);
                let task_types = input.task_types.as_ref().map(|v| {
                    v.iter()
                        .filter_map(|s| s.as_str().try_into().ok())
                        .collect::<Vec<ContentTaskType>>()
                });
                let payload = match task_types {
                    Some(task_types) => payload.with_tasks(&task_types),
                    None => payload,
                };
                content_base
                    .cancel_task(payload)
                    .await
                    .map_err(convert_anyhow_error)?;

                Ok(())
            })
        })
        .mutation("regenerate", |t| {
            t(|ctx: TCtx, input: TaskRedoRequestPayload| async move {
                let library = ctx.library()?;
                let asset_object_data = library
                    .prisma_client()
                    .asset_object()
                    .find_unique(asset_object::id::equals(input.asset_object_id))
                    .exec()
                    .await?
                    .ok_or(anyhow::anyhow!("not found"))
                    .map_err(convert_anyhow_error)?;

                library
                    .prisma_client()
                    .file_handler_task()
                    .delete_many(vec![file_handler_task::asset_object_id::equals(
                        input.asset_object_id,
                    )])
                    .exec()
                    .await?;

                // delete existed artifacts
                let content_base = ctx.content_base()?;
                let payload = DeletePayload::new(&asset_object_data.hash);
                content_base
                    .delete(payload)
                    .await
                    .map_err(convert_anyhow_error)?;

                let content_metadata: ContentMetadata = {
                    if let Some(v) = &asset_object_data.media_data {
                        serde_json::from_str(v).unwrap_or_default()
                    } else {
                        ContentMetadata::default()
                    }
                };

                generate_thumbnail(
                    &library,
                    &content_base,
                    &asset_object_data.hash,
                    &content_metadata,
                )
                .await?;

                process_asset(&library, &ctx, asset_object_data.id, None).await?;

                Ok(())
            })
        })
}
