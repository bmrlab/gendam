extern crate api_server; // 引入 lib.rs 里面的内容
use api_server::{
    ctx::default::{Ctx, Store},
    CtxStore, CtxWithLibrary,
    ShareInfo,
};
use axum::{http::request::Parts, routing::get};
use dotenvy::dotenv;
use p2p::Node;
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

    let temp_dir = std::env::temp_dir();
    let cache_dir = temp_dir.clone();
    // 本地开发环境, cache_dir 直接和 temp_dir 共享一个目录, 暂时不会有问题

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

    let node = Arc::new(Mutex::<Node<ShareInfo>>::new(
        p2p::Node::new().expect("create node error"),
    ));

    let ctx = Ctx::<Store>::new(local_data_root, resources_dir, temp_dir, cache_dir, store, node);

    let app: axum::Router = axum::Router::new()
        .route("/", get(|| async { "Hello 'rspc'!" }))
        .nest("/rspc", {
            rspc_axum::endpoint(router.clone(), {
                let ctx = ctx.clone();
                move |_parts: Parts| {
                    // tracing::info!("Client requested operation '{}'", parts.uri.path());
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
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal(ctx.clone()))
        .await
        .unwrap();
}

async fn shutdown_signal(ctx: impl CtxWithLibrary) {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };
    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            tracing::info!("Ctrl-C received, unload library and shut down...");
            let _ = ctx.unload_library().await;
            std::process::exit(0);
        },
        _ = terminate => {
            tracing::info!("Ctrl-C received, unload library and shut down...");
            let _ = ctx.unload_library().await;
            std::process::exit(0);
        },
    }
}
