use std::path::PathBuf;
use content_library::Library;
use rspc::{Rspc, Router};
use serde_json::json;
use tracing::error;
use crate::CtxWithLibrary;

pub fn get_routes<TCtx>() -> Router<TCtx>
where TCtx: CtxWithLibrary + Clone + Send + Sync + 'static
{
    Rspc::<TCtx>::new().router()
        .procedure(
            "list",
            Rspc::<TCtx>::new().query(|ctx, _input: ()| async move {
                let res = list_libraries(&ctx.get_local_data_root());
                serde_json::to_value::<Vec<String>>(res).unwrap()
            })
        )
        .procedure(
            "create",
            Rspc::<TCtx>::new().mutation(|ctx, title: String| async move {
                let library = create_library(&ctx.get_local_data_root(), title).await;
                json!({
                    "id": library.id,
                    "dir": library.dir,
                    // "artifacts_dir": library.artifacts_dir,
                    // "index_dir": library.index_dir,
                    // "db_url": library.db_url,
                })
            })
        )
        .procedure(
            "set_current_library",
            Rspc::<TCtx>::new().mutation(|ctx, library_id: String| async move {
                ctx.switch_current_library(&library_id);
                json!({ "status": "ok" })
            })
        )
        .procedure(
            "get_current_library",
            Rspc::<TCtx>::new().query(|ctx, _input: ()| async move {
                let library = ctx.load_library();
                Ok(library.id)
            })
        )
}

fn list_libraries(local_data_root: &PathBuf) -> Vec<String> {
    match local_data_root.join("libraries").read_dir() {
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
