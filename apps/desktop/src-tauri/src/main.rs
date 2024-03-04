// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use dotenvy::dotenv;
use tauri::Manager;
// use tracing::{debug, error};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};
use api_server::CtxWithLibrary;
use content_library::{
    load_library,
    Library,
};

#[derive(Clone)]
struct Ctx {
    local_data_root: PathBuf,
    resources_dir: PathBuf,
    store: Arc<Mutex<tauri_plugin_store::Store<tauri::Wry>>>,
}

impl CtxWithLibrary for Ctx {
    fn get_local_data_root(&self) -> PathBuf {
        self.local_data_root.clone()
    }
    fn get_resources_dir(&self) -> PathBuf {
        self.resources_dir.clone()
    }
    fn load_library(&self) -> Library {
        let mut store = self.store.lock().unwrap();
        let _ = store.load();
        let library_id = match store.get("current-library-id") {
            Some(value) => value.as_str().unwrap().to_owned(),
            None => String::from("default"),
        };
        let library = load_library(&self.local_data_root, &library_id);
        library
    }
}

#[tokio::main]
async fn main() {
    match dotenv() {
        Ok(path) => println!(".env read successfully from {}", path.display()),
        Err(e) => println!("Could not load .env file: {e}"),
    };
    init_tracing();

    let router = api_server::router::get_router::<Ctx>();

    tauri::Builder::default()
        .setup(|app| {
            #[cfg(debug_assertions)] // only include this code on debug builds
            {
                let window = app.get_window("main").unwrap();
                window.open_devtools();
                window.close_devtools();
            }

            Ok(())
        })
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(rspc::integrations::tauri::plugin(router, |app| {
            let local_data_root = app
                .app_handle()
                .path_resolver()
                .app_local_data_dir()
                .expect("failed to find local data dir");
            let resources_dir = app
                .app_handle()
                .path_resolver()
                .resolve_resource("resources")
                .expect("failed to find resources dir");
            let store = tauri_plugin_store::StoreBuilder::new(
                app.app_handle(),
                ".settings.json".parse().unwrap()
            ).build();
            Ctx {
                local_data_root,
                resources_dir,
                store: Arc::new(Mutex::new(store)),
            }
        }))
        .invoke_handler(tauri::generate_handler![
            greet,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
fn greet(name: &str) -> String {
    println!("Hello, {}, from Server!", name);
    format!("Hello, {}, in Client!", name)
}

fn init_tracing() {
    tracing_subscriber::registry()
        .with(
            // load filters from the `RUST_LOG` environment variable.
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "muse_desktop=info".into()),
        )
        .with(tracing_subscriber::fmt::layer().with_ansi(true))
        .init();
}
