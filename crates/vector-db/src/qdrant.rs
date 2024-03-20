use anyhow::bail;
pub use qdrant_client::client::QdrantClient;
use serde::{Deserialize, Serialize};
use std::{
    path::PathBuf,
    sync::{
        mpsc::{self, Sender},
        Arc,
    },
};
use tokio::sync::oneshot;
use tracing::{debug, error, info, warn};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct QdrantServerStorageConfig {
    storage_path: PathBuf,
    snapshots_path: PathBuf,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct QdrantServerServiceConfig {
    host: String,
    http_port: usize,
    grpc_port: usize,
}

impl Default for QdrantServerServiceConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".into(),
            http_port: 6333,
            grpc_port: 6334,
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct QdrantServerConfig {
    storage: QdrantServerStorageConfig,
    log_level: String,
    service: QdrantServerServiceConfig,
}

impl Default for QdrantServerConfig {
    fn default() -> Self {
        Self {
            storage: QdrantServerStorageConfig {
                storage_path: "./storage".into(),
                snapshots_path: "./snapshots".into(),
            },
            log_level: "INFO".into(),
            service: QdrantServerServiceConfig::default(),
        }
    }
}

enum QdrantServerPayload {
    Kill,
}

#[derive(Debug)]
pub struct QdrantParams {
    pub dir: PathBuf,
    pub http_port: Option<usize>,
    pub grpc_port: Option<usize>,
}

impl Into<QdrantServerServiceConfig> for QdrantParams {
    fn into(self) -> QdrantServerServiceConfig {
        let default = QdrantServerServiceConfig::default();

        QdrantServerServiceConfig {
            http_port: self.http_port.unwrap_or(default.http_port),
            grpc_port: self.grpc_port.unwrap_or(default.grpc_port),
            ..Default::default()
        }
    }
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
        // create config path for qdrant in storage_path
        let config_path = params.dir.join("config");
        std::fs::create_dir_all(&config_path)?;
        let config_path = config_path.join("config.yaml");

        debug!("qdrant config: {}", config_path.display());

        let storage_path = params.dir.join("storage");
        let snapshots_path = params.dir.join("snapshots");

        std::fs::create_dir_all(&storage_path)?;
        std::fs::create_dir_all(&snapshots_path)?;

        // write config into it
        let config = QdrantServerConfig {
            storage: QdrantServerStorageConfig {
                storage_path,
                snapshots_path,
            },
            service: params.into(),
            ..Default::default()
        };

        let yaml = serde_yaml::to_string(&config)?;
        std::fs::write(&config_path, yaml)?;

        debug!("qdrant reading config from {}", config_path.display());

        let (tx, rx) = mpsc::channel::<QdrantServerPayload>();

        let current_exe_path = std::env::current_exe().expect("failed to get current executable");
        let current_dir = current_exe_path.parent().expect("failed to get parent directory");
        let sidecar_path = current_dir.join("qdrant");
        match std::process::Command::new(sidecar_path)
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

        let url = format!(
            "http://{}:{}",
            config.service.host, config.service.grpc_port
        );
        let probe = format!(
            "http://{}:{}",
            config.service.host, config.service.http_port
        );

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
