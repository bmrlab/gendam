mod models;

use crate::CtxWithLibrary;
use content_library::{create_library, list_library_dirs};
use prisma_lib::raw;
use rspc::{Router, RouterBuilder};
use serde::{Deserialize, Serialize};
// use serde_json::json;
use specta::Type;
use std::path::PathBuf;

pub fn get_routes<TCtx>() -> RouterBuilder<TCtx>
where
    TCtx: CtxWithLibrary + Clone + Send + Sync + 'static,
{
    Router::<TCtx>::new().mutation("query", |t| {
        t(|ctx, _:()| async move {
            let library = ctx.library()?;
            match library.prisma_client()._query_raw::<MyDataType>(raw!(r#"select "table", "pk", "cid", "val", "col_version", "db_version", COALESCE("site_id", crsql_site_id()), "cl", "seq" from crsql_changes;"#)).exec().await {
                Ok(res) => {
                    tracing::info!("res: {:?}", res);
                }
                Err(e) => {
                    tracing::error!("error: {:?}", e);
                }
            }
            Ok("")
        })
    })
}

#[derive(Debug, Deserialize)]
struct MyDataType {
    table: String,
    pk: String,
    cid: String,
    val: std::any::Any,
    col_version: i32,
    db_version: i32,
    site_id: Option<String>,
    cl: i32,
    seq: i32,
}
