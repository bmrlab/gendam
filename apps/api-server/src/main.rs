extern crate api_server; // 引入 lib.rs 里面的内容
use api_server::{
    ctx::default::{Ctx, Store},
    CtxStore, CtxWithLibrary,
};
use axum::{http::request::Parts, routing::get};
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

    // let log_dir = env::var("LOCAL_LOG_DIR").unwrap();
    // analytics_tracing::init_tracing_to_file(log_dir.into());
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

    let mut default_store = Store::new(local_data_root.join("settings.json"));
    default_store.load().unwrap_or_else(|e| {
        tracing::warn!("Failed to load store: {:?}", e);
    });

    let cors = CorsLayer::new()
        .allow_methods(Any)
        .allow_headers(Any)
        .allow_origin(Any);

    let store = Arc::new(Mutex::new(default_store));
    let router = api_server::get_routes::<Ctx<Store>>().arced();
    let ctx = Ctx::<Store>::new(local_data_root, resources_dir, store);

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
