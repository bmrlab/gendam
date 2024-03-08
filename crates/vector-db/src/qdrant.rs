use anyhow::bail;
use serde::{Deserialize, Serialize};
use std::{
    path::{Path, PathBuf},
    sync::{
        mpsc::{self, Sender},
        Arc,
    },
};
use tokio::sync::{oneshot, Mutex};
use tracing::{debug, error, info};

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

struct QdrantServer {
    tx: Sender<QdrantServerPayload>,
    url: String,
    dir: PathBuf,
}

#[derive(Clone)]
pub struct QdrantChannel {
    tx: Sender<QdrantChannelPayload>,
}

pub enum QdrantChannelPayload {
    Update((QdrantParams, tokio::sync::oneshot::Sender<bool>)),
    GetCurrent(tokio::sync::oneshot::Sender<Option<String>>),
}

impl QdrantChannel {
    pub async fn new(resources_dir: impl AsRef<Path>) -> Self {
        let (tx, rx) = mpsc::channel::<QdrantChannelPayload>();
        let current_server: Option<QdrantServer> = None;
        let current_server = Arc::new(Mutex::new(current_server));

        let download = file_downloader::FileDownload::new(file_downloader::FileDownloadConfig {
            resources_dir: resources_dir.as_ref().to_path_buf(),
            ..Default::default()
        });

        let binary_file_path = download
            .download_if_not_exists("qdrant")
            .await
            .expect("failed to download qdrant");

        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("failed to build tokio runtime");

        std::thread::spawn(move || {
            let local = tokio::task::LocalSet::new();
            local.spawn_local(async move {
                loop {
                    match rx.recv() {
                        Ok(payload) => match payload {
                            QdrantChannelPayload::Update((params, tx)) => {
                                debug!("update params: {:?}", params);

                                let current = current_server.clone();
                                let mut current = current.lock().await;

                                if current.is_none() {
                                    // start new one
                                    match QdrantServer::new(
                                        binary_file_path.clone(),
                                        params.dir,
                                        params.http_port,
                                        params.grpc_port,
                                    )
                                    .await
                                    {
                                        Ok(server) => {
                                            *current = Some(server);
                                            tx.send(true).unwrap();
                                        }
                                        Err(e) => {
                                            error!("failed to start qdrant server: {}", e);
                                            tx.send(false).unwrap();
                                        }
                                    }
                                } else if current.as_ref().unwrap().dir != params.dir {
                                    // kill current one
                                    if let Some(server) = current.take() {
                                        server.tx.send(QdrantServerPayload::Kill).unwrap();
                                    }

                                    // start new one
                                    match QdrantServer::new(
                                        binary_file_path.clone(),
                                        params.dir,
                                        params.http_port,
                                        params.grpc_port,
                                    )
                                    .await
                                    {
                                        Ok(server) => {
                                            *current = Some(server);
                                            tx.send(true).unwrap();
                                        }
                                        Err(e) => {
                                            error!("failed to start qdrant server: {}", e);
                                            tx.send(false).unwrap();
                                        }
                                    }
                                } else {
                                    tx.send(true).unwrap();
                                }
                            }
                            QdrantChannelPayload::GetCurrent(tx) => {
                                if let Some(server) = current_server.lock().await.as_ref() {
                                    tx.send(Some(server.url.clone())).unwrap();
                                } else {
                                    tx.send(None).unwrap();
                                }
                            }
                        },
                        Err(e) => {
                            error!("error receive qdrant channel payload: {}", e);
                        }
                    }
                }
            });

            rt.block_on(local);
        });

        Self { tx }
    }

    pub async fn get_url(&self) -> String {
        let (tx, rx) = tokio::sync::oneshot::channel();

        self.tx.send(QdrantChannelPayload::GetCurrent(tx)).unwrap();
        rx.await.unwrap().unwrap()
    }

    pub async fn update(&self, params: QdrantParams) -> anyhow::Result<()> {
        let (tx, rx) = oneshot::channel();

        if let Err(e) = self.tx.send(QdrantChannelPayload::Update((params, tx))) {
            bail!("error send when update qdrant: {}", e);
        }

        match rx.await {
            Ok(res) => {
                if !res {
                    bail!("failed to update qdrant");
                }

                Ok(())
            }
            Err(e) => {
                bail!("error receive when update qdrant: {}", e);
            }
        }
    }
}

impl QdrantServer {
    pub async fn new(
        binary_file_path: impl AsRef<Path>,
        dir: impl AsRef<Path>,
        http_port: Option<usize>,
        grpc_port: Option<usize>,
    ) -> anyhow::Result<Self> {
        // create config path for qdrant in storage_path
        let config_path = dir.as_ref().join("config");
        std::fs::create_dir_all(&config_path)?;
        let config_path = config_path.join("config.yaml");

        debug!("qdrant config: {}", config_path.display());

        let storage_path = dir.as_ref().join("storage");
        let snapshots_path = dir.as_ref().join("snapshots");

        std::fs::create_dir_all(&storage_path)?;
        std::fs::create_dir_all(&snapshots_path)?;

        // write config into it
        let config = QdrantServerConfig {
            storage: QdrantServerStorageConfig {
                storage_path,
                snapshots_path,
            },
            service: QdrantServerServiceConfig {
                http_port: http_port.unwrap_or(6333),
                grpc_port: grpc_port.unwrap_or(6334),
                ..Default::default()
            },
            ..Default::default()
        };

        let yaml = serde_yaml::to_string(&config)?;
        std::fs::write(&config_path, yaml)?;

        debug!("qdrant reading config from {}", config_path.display());

        let (tx, rx) = mpsc::channel::<QdrantServerPayload>();

        match std::process::Command::new(binary_file_path.as_ref())
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
        let test_url = format!(
            "http://{}:{}",
            config.service.host, config.service.http_port
        );

        // use channel and select to make sure server is started
        let (tx1, rx1) = oneshot::channel();
        tokio::spawn(async move {
            loop {
                let resp = reqwest::get(test_url.clone()).await;
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
                debug!("qdrant started");
            }
        }

        Ok(Self {
            tx,
            url,
            dir: dir.as_ref().to_path_buf(),
        })
    }
}

impl Drop for QdrantServer {
    fn drop(&mut self) {
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
