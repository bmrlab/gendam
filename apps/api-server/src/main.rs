extern crate api_server; // 引入 lib.rs 里面的内容
use api_server::{
    ctx::default::{Ctx, Store},
    CtxStore, CtxWithLibrary,
};
use axum::{http::request::Parts, routing::get};
use content_library::{load_library, Library};
use dotenvy::dotenv;
use std::{
    env,
    path::Path,
    sync::{Arc, Mutex},
};
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
};

#[tokio::main]
async fn main() {
    match dotenv() {
        Ok(path) => println!(".env read successfully from {}", path.display()),
        Err(e) => println!("Could not load .env file: {e}"),
    };

    analytics_tracing::init_tracing_to_stdout();
    {
        // https://docs.rs/tracing/latest/tracing/struct.Span.html#in-asynchronous-code
        // Spans will be sent to the configured OpenTelemetry exporter
        // let root = tracing::span!(tracing::Level::INFO, "api-server", custom_field="custom value");
        // let _enter = root.enter();
        // tracing::error!("This event will be logged in the root span.");
    }

    let local_data_root = match env::var("LOCAL_DATA_DIR") {
        Ok(path) => Path::new(&path).to_path_buf(),
        Err(_e) => {
            // #[cfg(not(debug_assertions))]
            // {}
            panic!("'$LOCAL_DATA_DIR' is not set ({})", _e)
        }
    };
    std::fs::create_dir_all(&local_data_root).unwrap();

    let resources_dir = match env::var("LOCAL_RESOURCES_DIR") {
        Ok(path) => Path::new(&path).to_path_buf(),
        Err(_e) => {
            panic!("'$LOCAL_RESOURCES_DIR' is not set ({})", _e)
        }
    };

    let current_library = Arc::new(Mutex::<Option<Library>>::new(None));

    let mut default_store = Store::new(local_data_root.join("settings.json"));
    default_store.load().unwrap_or_else(|e| {
        tracing::warn!("Failed to load store: {:?}", e);
    });

    // try to kill current qdrant server, if any
    match default_store.get("current-qdrant-pid") {
        Some(pid) => {
            if let Ok(pid) = pid.parse::<i32>() {
                if vector_db::kill_qdrant_server(pid).is_err() {
                    tracing::warn!("Failed to kill qdrant server according to store");
                };
            }
            let _ = default_store.delete("current-qdrant-pid");
        }
        _ => {}
    }

    if default_store.save().is_err() {
        tracing::warn!("Failed to save store");
    }

    if let Some(library_id) = default_store.get("current-library-id") {
        match load_library(&local_data_root, &library_id).await {
            Ok(library) => {
                let pid = library.qdrant_server_info();
                current_library.lock().unwrap().replace(library);
                let _ = default_store.insert("current-qdrant-pid", &pid.to_string());
                if default_store.save().is_err() {
                    tracing::warn!("Failed to save store");
                }
            }
            Err(e) => {
                tracing::error!("Failed to load library: {:?}", e);
                let _ = default_store.delete("current-library-id");
                let _ = default_store.save();
            }
        };
    }

    let cors = CorsLayer::new()
        .allow_methods(Any)
        .allow_headers(Any)
        .allow_origin(Any);

    let store = Arc::new(Mutex::new(default_store));
    let router = api_server::get_routes::<Ctx<Store>>().arced();
    let ctx = Ctx::<Store>::new(local_data_root, resources_dir, store, current_library);

    let ctx_clone = ctx.clone();
    tokio::spawn(async move {
        ctx_clone.trigger_unfinished_tasks().await;
    });

    let app: axum::Router = axum::Router::new()
        .route("/", get(|| async { "Hello 'rspc'!" }))
        .nest("/rspc", {
            rspc_axum::endpoint(router.clone(), {
                move |parts: Parts| {
                    tracing::info!("Client requested operation '{}'", parts.uri.path());
                    // 不能每次 new 而应该是 clone，这样会保证 ctx 里面的每个元素每次只是新建了引用
                    ctx.clone()
                }
            })
        })
        // .nest_service("/artifacts", ServeDir::new(local_data_dir.clone()))
        .nest_service("/file/localhost", ServeDir::new("/"))
        .layer(cors);

    let addr = "[::]:3001".parse::<std::net::SocketAddr>().unwrap(); // This listens on IPv6 and IPv4
    tracing::debug!("Listening on http://{}/rspc/version", addr);
    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app)
        .await
        .unwrap();
}
