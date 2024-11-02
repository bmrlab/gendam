use super::super::ctx::traits::CtxWithLibrary;
use axum::{extract::Path as PathParams, response::Redirect, routing::get, Router};
use tower_http::services::ServeDir;

pub fn get_localhost_routes<TCtx>(ctx: TCtx) -> Router
where
    TCtx: CtxWithLibrary + Clone + Send + Sync + 'static,
{
    Router::new()
        .route("/_storage/localhost/asset_object/:hash/file", {
            let ctx = ctx.clone();
            get(|PathParams(hash): PathParams<String>| async move {
                match ctx.library() {
                    Ok(library) => {
                        let file_full_path_on_disk = library.file_full_path_on_disk(hash.as_str());
                        let new_path = format!(
                            "/_unsafe/localhost{}",
                            file_full_path_on_disk.to_string_lossy().as_ref()
                        );
                        Ok(Redirect::permanent(&new_path))
                    }
                    Err(e) => {
                        tracing::error!("Failed to load library: {:?}", e);
                        Err("Failed to load library")
                    }
                }
            })
        })
        .route("/_storage/localhost/asset_object/:hash/artifacts/:rest", {
            let ctx = ctx.clone();
            get(
                |PathParams((hash, rest)): PathParams<(String, String)>| async move {
                    match ctx.library() {
                        Ok(library) => {
                            let artifacts_dir_path_on_disk =
                                library._absolute_artifacts_dir(hash.as_str());
                            let file_full_path_on_disk = artifacts_dir_path_on_disk.join(rest);
                            let new_path = format!(
                                "/_unsafe/localhost{}",
                                file_full_path_on_disk.to_string_lossy().as_ref()
                            );
                            Ok(Redirect::permanent(&new_path))
                        }
                        Err(e) => {
                            tracing::error!("Failed to load library: {:?}", e);
                            Err("Failed to load library")
                        }
                    }
                },
            )
        })
        // .nest_service("/artifacts", ServeDir::new(local_data_dir.clone()))
        .nest_service("/_unsafe/localhost/", ServeDir::new("/"))
}
