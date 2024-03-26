use crate::CtxWithLibrary;
use content_library::{create_library_with_title, list_libraries, get_library_settings};
use rspc::{Router, RouterBuilder};
use serde_json::json;

pub fn get_routes<TCtx>() -> RouterBuilder<TCtx>
where
    TCtx: CtxWithLibrary + Clone + Send + Sync + 'static,
{
    Router::<TCtx>::new()
        .query("list", |t| {
            t(|ctx, _input: ()| async move {
                list_libraries(&ctx.get_local_data_root())
                // serde_json::to_value::<Vec<serde_json::Value>>(res).unwrap()
            })
        })
        .mutation("create", |t| {
            t(|ctx, title: String| async move {
                let library = create_library_with_title(
                    &ctx.get_local_data_root(), title.as_str()
                ).await;
                json!({
                    "id": library.id,
                    "dir": library.dir,
                })
            })
        })
        .mutation("set_current_library", |t| {
            t(|ctx, library_id: String| async move {
                ctx.switch_current_library(&library_id).await;
                json!({ "status": "ok" })
            })
        })
        .query("get_current_library", |t| {
            t(|ctx, _input: ()| async move {
                let library = ctx.library()?;
                let settings = get_library_settings(&library.dir);
                Ok(serde_json::json!({
                    "id": library.id,
                    "settings": settings,
                }))
            })
        })
}
