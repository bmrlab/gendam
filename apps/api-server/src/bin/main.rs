use api_server::exports::standalone;
#[tokio::main]
async fn main() {
    if let Err(e) = standalone::start_server().await {
        eprintln!("Error starting standalone server: {:?}", e);
    }
}
