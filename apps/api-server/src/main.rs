extern crate api_server;  // 引入 lib.rs 里面的内容
use api_server::{Ctx, router};

use std::net::SocketAddr;
use rspc::integrations::httpz::Request;
use axum::{
    routing::get,
    // Router,
};
use tower_http::cors::{Any, CorsLayer};

#[tokio::main]
async fn main() {
    let router = router::get_router();

    let cors = CorsLayer::new()
        .allow_methods(Any)
        .allow_headers(Any)
        .allow_origin(Any);
    let app: axum::Router = axum::Router::new()
        .route("/", get(|| async { "Hello 'rspc'!" }))
        .nest(
            "/rspc",
            router
                .clone()
                .endpoint(|req: Request| {
                    println!("Client requested operation '{}'", req.uri().path());
                    Ctx {
                        x_demo_header: req
                            .headers()
                            .get("X-Demo-Header")
                            .map(|v| v.to_str().unwrap().to_string()),
                    }
                })
                .axum()
        )
        .layer(cors);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3001));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
