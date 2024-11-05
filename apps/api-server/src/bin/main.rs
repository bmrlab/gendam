use api_server::exports::standalone;
#[tokio::main]
async fn main() {
    standalone::start_server().await;
}
