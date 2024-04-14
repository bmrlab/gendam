use std::path::PathBuf;
use crate::CtxWithLibrary;
use content_library::{create_library, list_library_dirs};
use rspc::{Router, RouterBuilder};
use serde_json::json;
use serde::{Serialize, Deserialize};
use specta::Type;

// libraries/[uuid as library id]/settings.json
const LIBRARY_SETTINGS_FILE_NAME: &str = "settings.json";

#[derive(Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct LibrarySettings {
    pub title: String
}

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
                Ok(LibrarySettings {
                    title: settings["title"].as_str().unwrap_or("Untitled").to_string(),
                })
            })
        })
        .mutation("update_library_settings", |t| {
            t(|ctx, input: LibrarySettings| async move {
                let library = ctx.library()?;
                set_library_settings(
                    &library.dir,
                    json!({
                        "title": input.title
                    })
                );
                Ok(())
            })
        })
        .mutation("set_current_library", |t| {
            t(|ctx, library_id: String| async move {
                ctx.switch_current_library(&library_id).await;
                json!({ "status": "ok" })
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

pub fn get_library_settings(library_dir: &PathBuf) -> serde_json::Value {
    match std::fs::File::open(library_dir.join(LIBRARY_SETTINGS_FILE_NAME)) {
        Ok(file) => {
            let reader = std::io::BufReader::new(file);
            match serde_json::from_reader(reader) {
                Ok(values) => values,
                Err(e) => {
                    tracing::error!("Failed to read file: {}", e);
                    serde_json::json!({ "title": "Untitled" })
                }
            }
        }
        Err(e) => {
            tracing::error!("Failed to open library's settings.json, {}", e);
            serde_json::json!({ "title": "Untitled" })
        }
    }
}

pub fn set_library_settings(library_dir: &PathBuf, settings: serde_json::Value) {
    // create or update to library_dir.join(LIBRARY_SETTINGS_FILE_NAME)
    match std::fs::File::create(library_dir.join(LIBRARY_SETTINGS_FILE_NAME)) {
        Ok(file) => {
            if let Err(e) = serde_json::to_writer(file, &settings) {
                tracing::error!("Failed to write file: {}", e);
            }
        }
        Err(e) => {
            tracing::error!("Failed to create file: {}", e);
        }
    };
}
