extern crate api_server;  // 引入 lib.rs 里面的内容
use api_server::{Ctx, router};
use dotenvy::dotenv;
use std::{env, net::SocketAddr, path::Path};
use rspc::integrations::httpz::Request;
use axum::routing::get;
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
};
use tracing::{info, debug, error};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    match dotenv() {
        Ok(path) => info!(".env read successfully from {}", path.display()),
        Err(e) => error!("Could not load .env file: {e}"),
    };
    init_tracing();  // should be after dotenv() so RUST_LOG in .env file will be loaded
    // debug!("test debug output");
    let local_data_dir = match env::var("LOCAL_DATA_DIR") {
		Ok(path) => Path::new(&path).to_path_buf(),
		Err(_e) => {
			// #[cfg(not(debug_assertions))]
			// {
				panic!("'$LOCAL_DATA_DIR' is not set ({})", _e)
			// }
		}
	};
    let resources_dir = local_data_dir.join("resources").to_str().unwrap().to_owned();
    let resources_dir = Path::new(&resources_dir).to_path_buf();

    let router = router::get_router();

    let cors = CorsLayer::new()
        .allow_methods(Any)
        .allow_headers(Any)
        .allow_origin(Any);
    let app: axum::Router = axum::Router::new()
        .route("/", get(|| async { "Hello 'rspc'!" }))
        .nest(
            "/rspc",
            {
                let local_data_dir = local_data_dir.clone();
                router.clone().endpoint(|req: Request| {
                    println!("Client requested operation '{}'", req.uri().path());
                    Ctx {
                        x_demo_header: req
                            .headers()
                            .get("X-Demo-Header")
                            .map(|v| v.to_str().unwrap().to_string()),
                        local_data_dir,
                        resources_dir,
                    }
                }).axum()
            }
        )
        .nest_service("/assets", ServeDir::new(local_data_dir.clone()))
        .nest_service("/contents", ServeDir::new("/"))
        .layer(cors);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3001));
    debug!("Listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

fn init_tracing() {
    tracing_subscriber::registry()
        .with(
            // load filters from the `RUST_LOG` environment variable.
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "api_server=info".into()),
        )
        .with(tracing_subscriber::fmt::layer().with_ansi(true))
        .init();
}
