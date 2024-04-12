extern crate api_server; // 引入 lib.rs 里面的内容
use api_server::{
    ctx::default::{Ctx, Store},
    CtxStore,
};
use axum::{http::request::Parts, routing::get};
use content_library::{load_library, Library};
use dotenvy::dotenv;
use std::{
    env,
    net::{Ipv4Addr, SocketAddr},
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
    match (
        default_store.get("current-qdrant-pid"),
        default_store.get("current-qdrant-http-port"),
    ) {
        (Some(pid), Some(port)) => {
            match (pid.parse(), port.parse::<u16>()) {
                (Ok(pid), Ok(port)) => {
                    if vector_db::kill_qdrant_server(
                        pid,
                        SocketAddr::new(std::net::IpAddr::V4(Ipv4Addr::LOCALHOST), port),
                    )
                    .is_err()
                    {
                        tracing::warn!("Failed to kill qdrant server according to store");
                    };
                }
                _ => {
                    tracing::warn!("invalid qdrant config, skipping killing qdrant server");
                }
            }
            let _ = default_store.delete("current-qdrant-pid");
            let _ = default_store.delete("current-qdrant-http-port");
            let _ = default_store.delete("current-qdrant-grpc-port");
        }
        _ => {}
    }

    if let Some(library_id) = default_store.get("current-library-id") {
        match load_library(&local_data_root, &library_id).await {
            Ok(library) => {
                let (pid, http_port, grpc_port) = library.qdrant_server_info();
                current_library.lock().unwrap().replace(library);
                let _ = default_store.insert("current-qdrant-pid", &pid.to_string());
                let _ = default_store.insert("current-qdrant-http-port", &http_port.to_string());
                let _ = default_store.insert("current-qdrant-grpc-port", &grpc_port.to_string());
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
