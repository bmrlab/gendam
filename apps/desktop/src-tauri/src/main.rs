// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use dotenvy::dotenv;
use tauri::Manager;
use tracing::info;
use api_server::{
    task_queue::{init_task_pool, TaskPayload},
    CtxWithLibrary,
};
use content_library::{load_library, upgrade_library_schemas, Library};
use tokio::sync::broadcast;
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
    pin::Pin,
    boxed::Box,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use vector_db::QdrantChannel;

#[derive(Clone)]
struct Ctx {
    local_data_root: PathBuf,
    resources_dir: PathBuf,
    store: Arc<Mutex<tauri_plugin_store::Store<tauri::Wry>>>,
    current_library: Arc<Mutex<Option<Library>>>,
    tx: Arc<broadcast::Sender<TaskPayload>>,
    qdrant_channel: Arc<QdrantChannel>,
}

impl CtxWithLibrary for Ctx {
    fn get_local_data_root(&self) -> PathBuf {
        self.local_data_root.clone()
    }
    fn get_resources_dir(&self) -> PathBuf {
        self.resources_dir.clone()
    }
    fn library(&self) -> Result<Library, rspc::Error> {
        match self.current_library.lock().unwrap().as_ref() {
            Some(library) => Ok(library.clone()),
            None => Err(rspc::Error::new(
                rspc::ErrorCode::BadRequest,
                String::from("No current library is set"),
            )),
        }
    }
    fn switch_current_library<'async_trait>(&'async_trait self, library_id: &'async_trait str)
        -> Pin<Box<dyn std::future::Future<Output = ()> + Send + 'async_trait>>
    where
        Self: Sync + 'async_trait,
    {
        let mut store = self.store.lock().unwrap();
        let _ = store.insert(String::from("current-library-id"), library_id.into());
        let _ = store.save();
        // try to load library, but this is not necessary
        let _ = store.load();
        if let Some(value) = store.get(String::from("current-library-id")) {
            let library_id = value.as_str().unwrap().to_owned();
            return Box::pin(async move {
                let library = load_library(&self.local_data_root, &library_id).await;
                self.current_library.lock().unwrap().replace(library);
                info!("Current library switched to {}", library_id);
            });
        } else {
            // 这里实际上不可能被执行，除非 settings.json 数据有问题
            return Box::pin(async move {
                self.current_library.lock().unwrap().take();
                info!("Current library is unset");
            });
        }
    }
    fn get_task_tx(&self) -> Arc<broadcast::Sender<TaskPayload>> {
        Arc::clone(&self.tx)
    }
    fn get_qdrant_channel(&self) -> Arc<QdrantChannel> {
        Arc::clone(&self.qdrant_channel)
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

    upgrade_library_schemas(&local_data_root).await;

    let store = Arc::new(Mutex::new(
        tauri_plugin_store::StoreBuilder::new(
            window.app_handle(),
            "settings.json".parse().unwrap(),
        )
        .build(),
    ));
    let current_library = Arc::new(Mutex::<Option<Library>>::new(None));
    {
        let mut store_mut = store.lock().unwrap();
        let _ = store_mut.load();
        if let Some(value) = store_mut.get("current-library-id") {
            let library_id = value.as_str().unwrap().to_owned();
            let library = load_library(&local_data_root, &library_id).await;
            current_library.lock().unwrap().replace(library);
        }
    }

    // app.app_handle()
    window
        .app_handle()
        .plugin(tauri_plugin_store::Builder::default().build())
        .expect("failed to add store plugin");

    let tx = init_task_pool();
    let router = api_server::router::get_router::<Ctx>();

    let qdrant_channel = QdrantChannel::new(&resources_dir).await;
    let qdrant_channel = Arc::new(qdrant_channel);

    window
        .app_handle()
        .plugin(rspc::integrations::tauri::plugin(router, move |_window| {
            Ctx {
                local_data_root: local_data_root.clone(),
                resources_dir: resources_dir.clone(),
                store: store.clone(),
                current_library: current_library.clone(),
                tx: tx.clone(),
                qdrant_channel: qdrant_channel.clone(),
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

// fn init_tracing() {
//     use std::fs::File;
//     use tracing_subscriber::Layer;
//     // create debug.log in current directory
//     let file = File::create("debug.log");
//     let file = match file {
//         Ok(file) => file,
//         Err(error) => panic!("Error: {:?}",error)
//     };
//     tracing_subscriber::registry()
//         .with(
//             tracing_subscriber::EnvFilter::try_from_default_env()
//                 .unwrap_or_else(|_| "debug".into())
//                 // .unwrap_or_else(|_| "muse_desktop=info".into()),
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
