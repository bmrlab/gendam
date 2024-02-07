// use super::router::get_router;
use axum::{
    routing::get,
    // Router,
};

#[tokio::main]
async fn main() {
    let router = api_server::router::get_router();
    let router = router.arced();

    let app = axum::Router::new()
        .route("/", get(|| async { "Hello 'rspc'!" }))
        .route("/rspc/:id", router.endpoint(|| ()).axum());

    let port = String::from("3000");
    let host = String::from("0.0.0.0");
    let socket_addr = format!("{}:{}", host, port);
    let listener = tokio::net::TcpListener::bind(&socket_addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
