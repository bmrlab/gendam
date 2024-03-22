use anyhow::bail;
pub use qdrant_client::client::QdrantClient;
use std::{
    path::PathBuf,
    sync::{
        mpsc::{self, Sender},
        Arc,
    },
};
use tokio::sync::oneshot;
use tracing::{debug, error, info, warn};

enum QdrantServerPayload {
    Kill,
}

#[derive(Debug)]
pub struct QdrantParams {
    pub dir: PathBuf,
    pub http_port: Option<usize>,
    pub grpc_port: Option<usize>,
}

#[derive(Clone)]
pub struct QdrantServer {
    tx: Sender<QdrantServerPayload>,
    client: Arc<QdrantClient>,
}

impl std::fmt::Debug for QdrantServer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QdrantServer").finish()
    }
}

impl QdrantServer {
    pub async fn new(params: QdrantParams) -> anyhow::Result<Self> {
        let host = "127.0.0.1";
        let http_port = params.http_port.unwrap_or(6333);
        let grpc_port = params.grpc_port.unwrap_or(6334);

        // create config path for qdrant in storage_path
        let config_path = params.dir.join("config");
        std::fs::create_dir_all(&config_path)?;
        let config_path = config_path.join("config.yaml");

        debug!("qdrant config: {}", config_path.display());

        if !config_path.exists() {
            let storage_path = params.dir.join("storage");
            let snapshots_path = params.dir.join("snapshots");

            std::fs::create_dir_all(&storage_path)?;
            std::fs::create_dir_all(&snapshots_path)?;

            let yaml = format!(
                r#"log_level: INFO
service:
  host: 0.0.0.0
  http_port: 6333
  grpc_port: 6334
storage:
  storage_path: {}
  snapshots_path: {}
  on_disk_payload: false
  optimizer:
    default_segment_number: 1
  performance:
    max_search_threads: 1
"#,
                storage_path.display(),
                snapshots_path.display()
            );
            std::fs::write(&config_path, yaml)?;
        }

        debug!("qdrant reading config from {}", config_path.display());

        let (tx, rx) = mpsc::channel::<QdrantServerPayload>();

        let current_exe_path = std::env::current_exe().expect("failed to get current executable");
        let current_dir = current_exe_path
            .parent()
            .expect("failed to get parent directory");
        let sidecar_path = current_dir.join("qdrant");
        match std::process::Command::new(sidecar_path)
            .env("QDRANT__SERVICE__HTTP_PORT", http_port.to_string())
            .env("QDRANT__SERVICE__GRPC_PORT", grpc_port.to_string())
            .args(["--config-path", config_path.to_str().expect("invalid path")])
            .spawn()
        {
            Ok(mut process) => {
                std::thread::spawn(move || loop {
                    if let Ok(action) = rx.recv() {
                        match action {
                            QdrantServerPayload::Kill => {
                                process.kill().unwrap();
                            }
                        }
                    }
                });
            }
            Err(e) => {
                bail!("failed to spawn qdrant: {}", e);
            }
        }

        let url = format!("http://{}:{}", host, grpc_port);
        let probe = format!("http://{}:{}", host, http_port);

        // use channel and select to make sure server is started
        let (tx1, rx1) = oneshot::channel();
        tokio::spawn(async move {
            loop {
                let resp = reqwest::get(probe.clone()).await;
                if let Ok(resp) = resp {
                    if resp.status() == reqwest::StatusCode::OK {
                        let _ = tx1.send(());
                        break;
                    }
                }
                // check for every 2s
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            }
        });
        tokio::select! {
            // timeout for 30s
            _ = tokio::time::sleep(std::time::Duration::from_secs(30)) => {
                bail!("qdrant start timeout");
            }
            _ = rx1 => {
                info!("qdrant started");
            }
        }

        let client = QdrantClient::from_url(&url).build()?;

        Ok(Self {
            tx,
            client: Arc::new(client),
        })
    }

    pub fn get_client(&self) -> Arc<QdrantClient> {
        self.client.clone()
    }
}

impl Drop for QdrantServer {
    fn drop(&mut self) {
        warn!("qdrant server dropped");

        match self.tx.send(QdrantServerPayload::Kill) {
            Ok(_) => {
                info!("qdrant successfully killed");
            }
            Err(e) => {
                error!("failed to kill qdrant: {}", e);
            }
        }
    }
}
