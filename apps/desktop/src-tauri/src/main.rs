// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use api_server::{ctx::default::Ctx, CtxWithLibrary};
use content_library::{load_library, Library};
use dotenvy::dotenv;
use serde_json::json;
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};
use tauri::Manager;

mod store;
use store::Store;

fn validate_app_version(app_handle: tauri::AppHandle, local_data_root: &PathBuf) {
    const VERSION_SHOULD_GTE: usize = 2;
    let mut tauri_store =
        tauri_plugin_store::StoreBuilder::new(app_handle, "settings.json".parse().unwrap()).build();
    tauri_store.load().unwrap_or_else(|e| {
        tracing::warn!("Failed to load tauri store: {:?}", e);
    });
    let version: usize = match tauri_store.get("version") {
        Some(value) => value.as_str().unwrap_or("").to_string(),
        None => "".to_string(),
    }
    .parse()
    .unwrap_or(0);
    if version < VERSION_SHOULD_GTE {
        // check if libraries exists, if true, move it to archived/libraries
        let libraries_dir = local_data_root.join("libraries");
        if libraries_dir.exists() {
            let archived_dir = local_data_root.join("archived");
            std::fs::create_dir_all(&archived_dir).unwrap();
            std::fs::rename(
                &libraries_dir,
                archived_dir.join(format!("libraries-{}", chrono::Utc::now().timestamp())),
            )
            .unwrap();
        }
        tauri_store.delete("current-library-id").unwrap();
        tauri_store
            .insert("version".to_string(), VERSION_SHOULD_GTE.to_string().into())
            .unwrap_or_else(|e| tracing::warn!("Failed to insert version to tauri store: {:?}", e));
        tauri_store.save().unwrap_or_else(|e| {
            tracing::warn!("Failed to save tauri store: {:?}", e);
        });
    }
}

#[tokio::main]
async fn main() {
    match dotenv() {
        Ok(path) => println!(".env read successfully from {}", path.display()),
        Err(e) => println!("Could not load .env file: {e}"),
    };

    let app = tauri::Builder::default()
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

    let app = app
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    #[cfg(not(debug_assertions))]
    {
        /*
         * macos log dir
         * ~/Library/Logs/cc.musedam.local/app.log
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
        // let root = tracing::span!(tracing::Level::INFO, "muse-desktop", custom_field="custom value");
        // let _enter = root.enter();
        // tracing::error!("This event will be logged in the root span.");
    }

    let window = app.get_window("main").unwrap();
    let local_data_root = window
        .app_handle()
        .path_resolver()
        .app_local_data_dir()
        .expect("failed to find local data dir");
    let resources_dir = window
        .app_handle()
        .path_resolver()
        .resolve_resource("resources")
        .expect("failed to find resources dir");

    // app.app_handle()
    window
        .app_handle()
        .plugin(tauri_plugin_store::Builder::default().build())
        .expect("failed to add store plugin");
    // TODO: 需要确认下 tauri_plugin_store 这个 plugin 是不是需要，如果网页上不用，应该不需要

    validate_app_version(window.app_handle(), &local_data_root);

    let current_library = Arc::new(Mutex::<Option<Library>>::new(None));

    let mut tauri_store = tauri_plugin_store::StoreBuilder::new(
        window.app_handle(),
        "settings.json".parse().unwrap(),
    )
    .build();
    tauri_store.load().unwrap_or_else(|e| {
        tracing::warn!("Failed to load tauri store: {:?}", e);
    });

    // try to kill current qdrant server, if any
    match tauri_store.get("current-qdrant-pid") {
        Some(pid) => {
            if let Some(pid) = pid.as_str() {
                if vector_db::kill_qdrant_server(pid.parse().unwrap()).is_err() {
                    tracing::warn!("Failed to kill qdrant server according to store");
                };
            }
            let _ = tauri_store.delete("current-qdrant-pid");
            let _ = tauri_store.save();
        }
        _ => {}
    }

    if let Some(value) = tauri_store.get("current-library-id") {
        let library_id = value.as_str().unwrap().to_owned();
        match load_library(&local_data_root, &library_id).await {
            Ok(library) => {
                let pid = library.qdrant_server_info();
                current_library.lock().unwrap().replace(library);
                // 注意这里要插入字符串类型的 pid
                let _ = tauri_store.insert("current-qdrant-pid".into(), json!(pid.to_string()));
                if tauri_store.save().is_err() {
                    tracing::warn!("Failed to save store");
                }
            }
            Err(e) => {
                tracing::error!("Failed to load library: {:?}", e);
                let _ = tauri_store.delete("current-library-id");
                let _ = tauri_store.save();
                // return;
            }
        };
    }

    window.on_window_event({
        let current_library = current_library.clone();
        move |e| {
            if let tauri::WindowEvent::Destroyed = e {
                if let Some(library) = current_library.lock().unwrap().take() {
                    drop(library);
                }
            }
        }
    });

    let store = Arc::new(Mutex::new(Store::new(tauri_store)));
    let router = api_server::get_routes::<Ctx<Store>>();
    let ctx = Ctx::<Store>::new(local_data_root, resources_dir, store, current_library);

    let ctx_clone = ctx.clone();
    tokio::spawn(async move {
        ctx_clone.trigger_unfinished_tasks().await;
    });

    window
        .app_handle()
        .plugin(rspc_tauri::plugin(router.arced(), move |_window| {
            // 不能每次 new 而应该是 clone，这样会保证 ctx 里面的每个元素每次只是新建了引用
            ctx.clone()
        }))
        .expect("failed to add rspc plugin");

    app.run(|_, _| {});
}

#[tauri::command]
fn greet(name: &str) -> String {
    println!("Hello, {}, from Server!", name);
    format!("Hello, {}, in Client!", name)
}
