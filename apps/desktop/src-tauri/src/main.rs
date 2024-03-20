// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use api_server::{
    task_queue::{init_task_pool, TaskPayload},
    CtxWithLibrary,
};
use content_library::{load_library, upgrade_library_schemas, Library};
use dotenvy::dotenv;
use std::{
    boxed::Box,
    path::PathBuf,
    pin::Pin,
    sync::{Arc, Mutex},
};
use tauri::Manager;
use tokio::sync::broadcast;
use tokio_util::sync::CancellationToken;
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Clone)]
struct Ctx {
    local_data_root: PathBuf,
    resources_dir: PathBuf,
    store: Arc<Mutex<tauri_plugin_store::Store<tauri::Wry>>>,
    current_library: Arc<Mutex<Option<Library>>>,
    tx: Arc<Mutex<broadcast::Sender<TaskPayload>>>,
    cancel_token: Arc<Mutex<CancellationToken>>,
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
    fn switch_current_library<'async_trait>(
        &'async_trait self,
        library_id: &'async_trait str,
    ) -> Pin<Box<dyn std::future::Future<Output = ()> + Send + 'async_trait>>
    where
        Self: Sync + 'async_trait,
    {
        // cancel all tasks
        self.cancel_token.lock().unwrap().cancel();
        let (tx, cancel_token) = init_task_pool();
        let mut old_tx = self.tx.lock().unwrap();
        let mut old_cancel_token = self.cancel_token.lock().unwrap();
        *old_tx = tx;
        *old_cancel_token = cancel_token;

        let mut store = self.store.lock().unwrap();
        let _ = store.insert(String::from("current-library-id"), library_id.into());
        let _ = store.save();
        // try to load library, but this is not necessary
        let _ = store.load();
        if let Some(value) = store.get(String::from("current-library-id")) {
            let library_id = value.as_str().unwrap().to_owned();
            return Box::pin(async move {
                let library = load_library(&self.local_data_root, &self.resources_dir, &library_id)
                    .await
                    .unwrap();
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
    fn get_task_tx(&self) -> Arc<Mutex<broadcast::Sender<TaskPayload>>> {
        self.tx.clone()
    }
}

#[tokio::main]
async fn main() {
    match dotenv() {
        Ok(path) => println!(".env read successfully from {}", path.display()),
        Err(e) => println!("Could not load .env file: {e}"),
    };

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
        .invoke_handler(tauri::generate_handler![greet,]);

    let app = app
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    init_tracing(app.path_resolver().app_log_dir().unwrap());

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
            let library = match load_library(&local_data_root, &resources_dir, &library_id).await {
                Ok(library) => library,
                Err(e) => {
                    error!("Failed to load library: {:?}", e);
                    return;
                }
            };
            current_library.lock().unwrap().replace(library);
        }
    }

    window.on_window_event({
        let current_library = current_library.clone();
        move |e| {
            if let tauri::WindowEvent::Destroyed = e {
                let library = current_library.lock().unwrap().take().unwrap();
                drop(library.qdrant_server);
            }
        }
    });

    // app.app_handle()
    window
        .app_handle()
        .plugin(tauri_plugin_store::Builder::default().build())
        .expect("failed to add store plugin");

    let (tx, cancel_token) = init_task_pool();
    let tx = Arc::new(Mutex::new(tx));
    let cancel_token = Arc::new(Mutex::new(cancel_token));
    let router = api_server::router::get_router::<Ctx>();

    window
        .app_handle()
        .plugin(rspc::integrations::tauri::plugin(router, move |_window| {
            Ctx {
                local_data_root: local_data_root.clone(),
                resources_dir: resources_dir.clone(),
                store: store.clone(),
                current_library: current_library.clone(),
                tx: tx.clone(),
                cancel_token: cancel_token.clone(),
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

fn init_tracing(log_dir: PathBuf) {
    #[cfg(debug_assertions)]
    {
        let _ = log_dir;
        tracing_subscriber::registry()
            .with(
                // load filters from the `RUST_LOG` environment variable.
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| "muse_desktop=debug".into()),
            )
            .with(tracing_subscriber::fmt::layer().with_ansi(true))
            .init();
    }
    #[cfg(not(debug_assertions))]
    {
        /*
         * see logs with cmd:
         * log stream --debug --predicate 'subsystem=="cc.musedam.local" and category=="default"'
         */
        let os_logger = tracing_oslog::OsLogger::new("cc.musedam.local", "default");
        /*
         * macos log dir
         * ~/Library/Logs/cc.musedam.local/app.log
         */
        if let Err(e) = std::fs::create_dir_all(&log_dir) {
            eprintln!("Failed to create log dir: {}", e);
            return;
        }
        let file = match std::fs::File::create(log_dir.join("app.log")) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("Failed to create log file: {}", e);
                return;
            }
        };
        let os_file_logger = tracing_subscriber::fmt::layer()
            .with_writer(Mutex::new(file))
            .with_ansi(false);
        tracing_subscriber::registry()
            .with(tracing_subscriber::EnvFilter::new("info"))
            .with(os_logger)
            .with(os_file_logger)
            .init();
    }
}
