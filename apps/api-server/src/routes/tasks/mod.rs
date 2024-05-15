use crate::CtxWithLibrary;
use prisma_client_rust::QueryError;
use rspc::{Router, RouterBuilder};
use serde::Deserialize;
use specta::Type;

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

fn sql_error(e: QueryError) -> rspc::Error {
    rspc::Error::new(
        rspc::ErrorCode::InternalServerError,
        format!("sql query failed: {}", e),
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
        .query("test", |t| {
            t(|_ctx: TCtx, _: ()| async move { "".to_string() })
        })
}
