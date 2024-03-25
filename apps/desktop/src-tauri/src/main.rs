// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use api_server::ctx::default::Ctx;
use content_library::{load_library, upgrade_library_schemas, Library};
use dotenvy::dotenv;
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};
use tauri::Manager;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod store;
use store::Store;

fn validate_app_version(app_handle: tauri::AppHandle, local_data_root: &PathBuf) {
    const VERSION_SHOULD_GTE: usize = 1;
    let mut tauri_store = tauri_plugin_store::StoreBuilder::new(
        app_handle,
        "settings.json".parse().unwrap(),
    ).build();
    tauri_store.load().unwrap_or_else(|e| {
        tracing::warn!("Failed to load tauri store: {:?}", e);
    });
    let version: usize = match tauri_store.get("version") {
        Some(value) => value.as_str().unwrap_or("").to_string(),
        None => "".to_string()
    }.parse().unwrap_or(0);
    if version < VERSION_SHOULD_GTE {
        // check if libraries exists, if true, move it to archived/libraries
        let libraries_dir = local_data_root.join("libraries");
        if libraries_dir.exists() {
            let archived_dir = local_data_root.join("archived");
            std::fs::create_dir_all(&archived_dir).unwrap();
            std::fs::rename(&libraries_dir, archived_dir.join("libraries")).unwrap();
        }
        tauri_store.delete("current-library-id").unwrap();
        tauri_store.insert(
            "version".to_string(),
            VERSION_SHOULD_GTE.to_string().into()
        ).unwrap();
        tauri_store.save().unwrap();
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

    validate_app_version(window.app_handle(), &local_data_root);
    upgrade_library_schemas(&local_data_root).await;

    let current_library = Arc::new(Mutex::<Option<Library>>::new(None));

    let mut tauri_store = tauri_plugin_store::StoreBuilder::new(
        window.app_handle(),
        "settings.json".parse().unwrap(),
    ).build();
    if let Err(e) = tauri_store.load() {
        tracing::error!("Failed to load store: {:?}", e);
        return;
    }

    if let Some(value) = tauri_store.get("current-library-id") {
        let library_id = value.as_str().unwrap().to_owned();
        let library = match load_library(&local_data_root, &library_id).await {
            Ok(library) => library,
            Err(e) => {
                tracing::error!("Failed to load library: {:?}", e);
                return;
            }
        };
        current_library.lock().unwrap().replace(library);
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

    let store = Arc::new(Mutex::new(Store::new(tauri_store)));
    let router = api_server::router::get_router::<Ctx<Store>>();
    let ctx = Ctx::<Store>::new(local_data_root, resources_dir, store, current_library);

    window
        .app_handle()
        .plugin(
            rspc_tauri::plugin(router.arced(), move |_window| {
                // 不能每次 new 而应该是 clone，这样会保证 ctx 里面的每个元素每次只是新建了引用
                ctx.clone()
            })
        )
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
