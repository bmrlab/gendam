use p2p::AutonatServer;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    analytics_tracing::init_tracing_to_stdout();
    Ok(AutonatServer::new(3002).run().await?)
}
