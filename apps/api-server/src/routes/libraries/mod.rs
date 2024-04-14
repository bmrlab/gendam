mod settings;
use settings::{get_library_settings, set_library_settings, LIBRARY_SETTINGS_FILE_NAME, LibrarySettings};
use std::path::PathBuf;
use crate::CtxWithLibrary;
use content_library::{create_library, list_library_dirs};
use rspc::{Router, RouterBuilder};
use serde_json::json;
use serde::Serialize;
use specta::Type;

pub fn get_routes<TCtx>() -> RouterBuilder<TCtx>
where
    TCtx: CtxWithLibrary + Clone + Send + Sync + 'static,
{
    Router::<TCtx>::new()
        .query("list", {
            #[derive(Serialize, Type)]
            #[serde(rename_all = "camelCase")]
            pub struct LibrariesListResult {
                pub id: String,
                pub dir: String,
                pub title: String,
            }
            |t| {
                t(|ctx, _input: ()| async move {
                    let library_dirs = list_library_dirs(&ctx.get_local_data_root());
                    library_dirs.into_iter().map(|(dir, id)| {
                        let title = match std::fs::File::open(
                            PathBuf::from(&dir).join(LIBRARY_SETTINGS_FILE_NAME)
                        ) {
                            Ok(file) => {
                                let reader = std::io::BufReader::new(file);
                                match serde_json::from_reader::<_, serde_json::Value>(reader) {
                                    Ok(values) => values["title"].as_str().unwrap_or("Untitled").to_string(),
                                    Err(_) => "Untitled".to_string()
                                }
                            }
                            Err(_) => "Untitled".to_string()
                        };
                        LibrariesListResult { id, dir, title }
                    }).collect::<Vec<LibrariesListResult>>()
                })
            }
        })
        .mutation("create", |t| {
            t(|ctx, title: String| async move {
                let library_dir = create_library(&ctx.get_local_data_root()).await;
                match std::fs::File::create(library_dir.join(LIBRARY_SETTINGS_FILE_NAME)) {
                    Ok(file) => {
                        let value = serde_json::json!({ "title": title });
                        if let Err(e) = serde_json::to_writer(file, &value) {
                            tracing::error!("Failed to write file: {}", e);
                        }
                    }
                    Err(e) => {
                        tracing::error!("Failed to create file: {}", e);
                    }
                };
                // json!({ "id": library.id, "dir": library.dir })
            })
        })
        .query("get_library_settings", |t| {
            t(|ctx, _input: ()| async move {
                let library = ctx.library()?;
                let settings = get_library_settings(&library.dir);
                Ok(settings)
            })
        })
        .mutation("update_library_settings", |t| {
            t(|ctx, input: LibrarySettings| async move {
                let library = ctx.library()?;
                set_library_settings(
                    &library.dir,
                    input
                );
                Ok(())
            })
        })
        .mutation("set_current_library", |t| {
            /*
             * TODO: 这里要改成先校验一下 ctx.library 是否为 None, 如果是, 直接 load 而不是 switch
             * 并且删除 switch 方法
             */
            t(|ctx, library_id: String| async move {
                ctx.switch_current_library(&library_id).await;
                json!({ "status": "ok" })
            })
        })
        .mutation("quit_current_library", |t| {
            t(|ctx, library_id: String| async move {
                /*
                 * TODO 如果这里不加一个参数直接用 _input: (), 会因参数校验失败而返回错误,
                 * 因为前端会发一个 payload: `{}`, 而不是空, 这个 issue 需要排查一下
                 * 现在就索性校验一下 library_id, 实际没啥用
                 */
                let library = ctx.library()?;
                if library.id != library_id {
                    return Err(rspc::Error::new(
                        rspc::ErrorCode::BadRequest,
                        String::from("The library is not the current library"),
                    ));
                }
                ctx.quit_current_library().await;
                Ok(json!({ "status": "ok" }))
            })
        })
        .query("get_current_library", {
            #[derive(Serialize, Type)]
            #[serde(rename_all = "camelCase")]
            pub struct CurrentLibraryResult {
                pub id: String,
                pub dir: String,
            }
            |t| {
                t(|ctx, _input: ()| async move {
                    let library = ctx.library()?;
                    Ok(CurrentLibraryResult {
                        id: library.id.clone(),
                        dir: library.dir.into_os_string().into_string().unwrap(),
                    })
                })
            }
        })
}
