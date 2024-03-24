use crate::CtxWithLibrary;
use content_library::Library;
use rspc::{Router, RouterBuilder};
use serde_json::json;
use std::path::PathBuf;
use tracing::error;

pub fn get_routes<TCtx>() -> RouterBuilder<TCtx>
where
    TCtx: CtxWithLibrary + Clone + Send + Sync + 'static,
{
    Router::<TCtx>::new()
        .query("list", |t| {
            t(|ctx, _input: ()| async move {
                let res = list_libraries(&ctx.get_local_data_root());
                serde_json::to_value::<Vec<String>>(res).unwrap()
            })
        })
        .mutation("create", |t| {
            t(|ctx, title: String| async move {
                let library = create_library(&ctx.get_local_data_root(), title).await;
                json!({
                    "id": library.id,
                    "dir": library.dir,
                    // "artifacts_dir": library.artifacts_dir,
                    // "index_dir": library.index_dir,
                    // "db_url": library.db_url,
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
                Ok(library.id)
            })
        })
}

fn list_libraries(local_data_root: &PathBuf) -> Vec<String> {
    let libraries_dir = local_data_root.join("libraries");
    if !libraries_dir.exists() {
        return vec![];
    }
    match libraries_dir.read_dir() {
        Ok(entries) => {
            let mut res = vec![];
            for entry in entries {
                match entry.as_ref() {
                    Ok(entry) => {
                        let path = entry.path();
                        let file_name = entry.file_name();
                        if path.is_dir() {
                            let file_name = file_name.to_str().unwrap().to_string();
                            res.push(file_name);
                        }
                    }
                    Err(e) => {
                        error!("Failed to read library dir: {}", e);
                        continue;
                    }
                };
            }
            res
        }
        Err(e) => {
            error!("Failed to read libraries dir: {}", e);
            vec![]
        }
    }
}

async fn create_library(local_data_root: &PathBuf, title: String) -> Library {
    let library = content_library::create_library_with_title(local_data_root, title.as_str()).await;
    return library;
}
