// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use api_server::exports::{
    ctx::{Ctx, CtxWithLibrary},
    get_rspc_routes,
    library::{load_library_exclusive_and_wait, unload_library_exclusive_and_wait},
};
use dotenvy::dotenv;
use global_variable::init_global_variables;
use std::sync::{Arc, Mutex};
use tauri::{http::Request, Manager};
mod storage;
mod store;

use crate::storage::protocol::storage_protocol_handler;
use crate::storage::state::StorageState;
use store::Store;

#[tokio::main]
async fn main() {
    match dotenv() {
        Ok(path) => println!(".env read successfully from {}", path.display()),
        Err(e) => println!("Could not load .env file: {e}"),
    };

    init_global_variables!();

    let app = {
        let app_builder = tauri::Builder::default()
            .register_uri_scheme_protocol("storage", move |app, request: &Request| {
                let state = app
                    .state::<Arc<tokio::sync::Mutex<StorageState>>>()
                    .inner()
                    .clone();
                storage_protocol_handler(state, request)
            })
            .setup(|_app| {
                #[cfg(debug_assertions)] // only include this code on debug builds
                {
                    let window = _app.get_window("main").unwrap();
                    window.open_devtools();
                    window.close_devtools();
                }
                Ok(())
            })
            .invoke_handler(tauri::generate_handler![greet,]);
        app_builder
            .build(tauri::generate_context!())
            .expect("error while building tauri application")
    };

    #[cfg(not(debug_assertions))]
    {
        /*
         * macos log dir
         * ~/Library/Logs/ai.gendam.desktop/app.log
         */
        let log_dir = app.path_resolver().app_log_dir().unwrap();
        analytics_tracing::init_tracing_to_file(log_dir);
    }
    #[cfg(debug_assertions)]
    {
        analytics_tracing::init_tracing_to_stdout();
    }
    {
        // https://docs.rs/tracing/latest/tracing/struct.Span.html#in-asynchronous-code
        // Spans will be sent to the configured OpenTelemetry exporter
        // let root = tracing::span!(tracing::Level::INFO, "gendam-desktop", custom_field="custom value");
        // let _enter = root.enter();
        // tracing::error!("This event will be logged in the root span.");
    }

    // tauri::async_runtime::spawn(async move {
    //     let mut node = p2p_clone.lock().unwrap().clone();
    //     node.start_p2p().await.unwrap();
    // });

    let window = app.get_window("main").unwrap();

    let store = {
        // app.app_handle()
        window
            .app_handle()
            .plugin(tauri_plugin_store::Builder::default().build())
            .expect("failed to add store plugin");
        // TODO: 需要确认下 tauri_plugin_store 这个 plugin 是不是需要，如果网页上不用，应该不需要
        // validate_app_version(window.app_handle(), &local_data_root);
        let mut tauri_store = tauri_plugin_store::StoreBuilder::new(
            window.app_handle(),
            "settings.json".parse().unwrap(),
        )
        .build();
        tauri_store.load().unwrap_or_else(|e| {
            tracing::warn!("Failed to load tauri store: {:?}", e);
        });
        Arc::new(Mutex::new(Store::new(tauri_store)))
    };

    let ctx = {
        let local_data_root = window
            .app_handle()
            .path_resolver()
            .app_local_data_dir()
            .expect("failed to find local data dir");
        std::fs::create_dir_all(&local_data_root).unwrap();

        let resources_dir = window
            .app_handle()
            .path_resolver()
            .resolve_resource("resources")
            .expect("failed to find resources dir");
        let temp_dir = std::env::temp_dir();
        let cache_dir = tauri::api::path::cache_dir().unwrap_or({
            tracing::error!("Failed to get cache dir");
            temp_dir.clone()
        });

        let p2p_node = p2p::Node::new().expect("Failed to create p2p node");
        let node = Arc::new(Mutex::new(p2p_node));
        // let p2p_clone = p2p.clone();

        Ctx::<Store>::new(
            local_data_root,
            resources_dir,
            temp_dir,
            cache_dir,
            store,
            node,
        )
    };

    // Load library if it is set in store, otherwise user should set it in the UI
    if let Some(library_id_in_store) = ctx.library_id_in_store() {
        if let Err(e) = load_library_exclusive_and_wait(ctx.clone(), library_id_in_store).await {
            panic!("Failed to load library: {:?}", e);
        }
    }

    // init opendal's storage state
    app.manage({
        let storage_state = StorageState::new(ctx.clone());
        Arc::new(tokio::sync::Mutex::new(storage_state))
    });

    window.on_window_event({
        let ctx = ctx.clone();
        let window = window.clone();
        move |e| {
            if let tauri::WindowEvent::Destroyed = e {
                // https://github.com/bmrlab/gendam/issues/10#issuecomment-2078827778
                tracing::info!("window destroyed");
                if let Ok(_library) = ctx.library() {
                    // drop(library);
                }
                tokio::runtime::Runtime::new().unwrap().block_on(async {
                    // let _ = ctx.unload_library().await;
                    let _ = unload_library_exclusive_and_wait(ctx.clone()).await;
                });
            }
            if let tauri::WindowEvent::CloseRequested { api, .. } = e {
                // Prevents the window from being closed.
                api.prevent_close();
                // Minimizes the window instead.
                window.minimize().unwrap();
            }
        }
    });

    window
        .app_handle()
        .plugin({
            let router = get_rspc_routes::<Ctx<Store>>().arced();
            rspc_tauri::plugin(router, move |_window| {
                // 不能每次 new 而应该是 clone，这样会保证 ctx 里面的每个元素每次只是新建了引用
                ctx.clone()
            })
        })
        .expect("failed to add rspc plugin");

    app.run(|_, _| {});
}

#[tauri::command]
fn greet(name: &str) -> String {
    println!("Hello, {}, from Server!", name);
    format!("Hello, {}, in Client!", name)
}

// #[allow(dead_code)]
// fn validate_app_version(app_handle: tauri::AppHandle, local_data_root: &std::path::PathBuf) {
//     // 目前先不需要调用这个, 这一次 release 不会用到旧的数据库
//     const VERSION_SHOULD_GTE: usize = 2;
//     let mut tauri_store =
//         tauri_plugin_store::StoreBuilder::new(app_handle, "settings.json".parse().unwrap()).build();
//     tauri_store.load().unwrap_or_else(|e| {
//         tracing::warn!("Failed to load tauri store: {:?}", e);
//     });
//     let version: usize = match tauri_store.get("version") {
//         Some(value) => value.as_str().unwrap_or("").to_string(),
//         None => "".to_string(),
//     }
//     .parse()
//     .unwrap_or(0);
//     if version < VERSION_SHOULD_GTE {
//         // check if libraries exists, if true, move it to archived/libraries
//         let libraries_dir = local_data_root.join("libraries");
//         if libraries_dir.exists() {
//             let archived_dir = local_data_root.join("archived");
//             std::fs::create_dir_all(&archived_dir).unwrap();
//             std::fs::rename(
//                 &libraries_dir,
//                 archived_dir.join(format!("libraries-{}", chrono::Utc::now().timestamp())),
//             )
//             .unwrap();
//         }
//         tauri_store.delete("current-library-id").unwrap();
//         tauri_store
//             .insert("version".to_string(), VERSION_SHOULD_GTE.to_string().into())
//             .unwrap_or_else(|e| tracing::warn!("Failed to insert version to tauri store: {:?}", e));
//         tauri_store.save().unwrap_or_else(|e| {
//             tracing::warn!("Failed to save tauri store: {:?}", e);
//         });
//     }
// }
