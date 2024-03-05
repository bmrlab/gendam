// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use dotenvy::dotenv;
use tauri::Manager;
// use tracing::{debug, error};
use api_server::{
    task_queue::{init_task_pool, TaskPayload},
    CtxWithLibrary,
};
use content_library::{load_library, upgrade_library_schemas, Library};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use vector_db::FaissIndex;

#[derive(Clone)]
struct Ctx {
    local_data_root: PathBuf,
    resources_dir: PathBuf,
    store: Arc<Mutex<tauri_plugin_store::Store<tauri::Wry>>>,
    tx: Arc<tokio::sync::broadcast::Sender<TaskPayload>>,
    index: FaissIndex,
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
    fn get_task_tx(&self) -> Arc<tokio::sync::broadcast::Sender<TaskPayload>> {
        Arc::clone(&self.tx)
    }
    fn get_index(&self) -> FaissIndex {
        self.index.clone()
    }
}

#[tokio::main]
async fn main() {
    match dotenv() {
        Ok(path) => println!(".env read successfully from {}", path.display()),
        Err(e) => println!("Could not load .env file: {e}"),
    };
    init_tracing();

    let app = tauri::Builder::default()
        .setup(|app| {
            #[cfg(debug_assertions)] // only include this code on debug builds
            {
                let window = app.get_window("main").unwrap();
                window.open_devtools();
                window.close_devtools();
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![greet,])
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

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
    let store = Arc::new(Mutex::new(
        tauri_plugin_store::StoreBuilder::new(
            window.app_handle(),
            ".settings.json".parse().unwrap(),
        )
        .build(),
    ));
    upgrade_library_schemas(&local_data_root).await;

    // app.app_handle()
    window
        .app_handle()
        .plugin(tauri_plugin_store::Builder::default().build())
        .expect("failed to add store plugin");

    let tx = init_task_pool();
    let index = FaissIndex::new();
    let router = api_server::router::get_router::<Ctx>();
    window
        .app_handle()
        .plugin(rspc::integrations::tauri::plugin(router, move |_window| {
            Ctx {
                local_data_root: local_data_root.clone(),
                resources_dir: resources_dir.clone(),
                store: store.clone(),
                tx: tx.clone(),
                index: index.clone(),
            }
        }))
        .expect("failed to add rspc plugin");

    app.run(|_, _| {});
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

// fn init_tracing_file() {
//     use std::fs::File;
//     let file = File::create("/Users/xddotcom/Library/Application Support/cc.musedam.local/debug.log");
//     let file = match file  {Ok(file) => file,Err(error) => panic!("Error: {:?}",error),};

//     tracing_subscriber::registry()
//         .with(
//             tracing_subscriber::EnvFilter::try_from_default_env()
//                 .unwrap_or_else(|_| "debug".into())
//         )
//         .with(
//             tracing_subscriber::fmt::layer()
//             .with_ansi(true)
//             .and_then(
//                 tracing_subscriber::fmt::layer()
//                 .with_writer(Arc::new(file))
//                 .with_ansi(false)
//             )
//         )
//         .init();
// }
