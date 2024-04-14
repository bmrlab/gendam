use crate::CtxWithLibrary;
use content_library::{create_library_with_title, list_libraries, get_library_settings, set_library_settings};
use rspc::{Router, RouterBuilder};
use serde_json::json;
use serde::{Serialize, Deserialize};
use specta::Type;

#[derive(Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct LibrarySettings {
    pub title: String
}

#[derive(Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Library {
    pub id: String,
    pub dir: String,
    pub settings: LibrarySettings,
}

pub fn get_routes<TCtx>() -> RouterBuilder<TCtx>
where
    TCtx: CtxWithLibrary + Clone + Send + Sync + 'static,
{
    Router::<TCtx>::new()
        .query("list", |t| {
            t(|ctx, _input: ()| async move {
                let res = list_libraries(&ctx.get_local_data_root());
                res.into_iter().map(|library| {
                    Library {
                        id: library["id"].as_str().unwrap().to_string(),
                        dir: library["dir"].as_str().unwrap().to_string(),
                        settings: LibrarySettings {
                            title: library["settings"]["title"].as_str().unwrap_or("Untitled").to_string(),
                        }
                    }
                }).collect::<Vec<Library>>()
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
        .query("get_current_library", |t| {
            t(|ctx, _input: ()| async move {
                let library = ctx.library()?;
                let settings = get_library_settings(&library.dir);
                Ok(Library {
                    id: library.id.clone(),
                    dir: library.dir.to_str().unwrap().to_string(),
                    settings: LibrarySettings {
                        title: settings["title"].as_str().unwrap_or("Untitled").to_string(),
                    }
                })
            })
        })
}
