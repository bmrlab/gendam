use anyhow::{anyhow, bail};
pub use qdrant_client::client::QdrantClient;
use rustix::process::{kill_process, Pid, Signal};
use std::io::BufRead;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::num::NonZeroI32;
use std::sync::mpsc::channel;
use std::thread::sleep;
use std::{path::PathBuf, sync::Arc};
use tokio::sync::oneshot;
use tracing::{debug, error, info, warn};

#[derive(Debug)]
pub struct QdrantParams {
    pub dir: PathBuf,
    pub http_port: Option<u16>,
    pub grpc_port: Option<u16>,
}

#[derive(Clone)]
pub struct QdrantServer {
    client: Arc<QdrantClient>,
    pid: Pid,
}

impl std::fmt::Debug for QdrantServer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QdrantServer").finish()
    }
}

impl QdrantServer {
    pub async fn new(params: QdrantParams) -> anyhow::Result<Self> {
        debug!("qdrant params: {:?}", params);

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

            // port info will be overwritten by environment variables
            // it's ok to just write default ones here
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

        let current_exe_path = std::env::current_exe().expect("failed to get current executable");
        let sidecar_path = current_exe_path.with_file_name("qdrant");
        let process = std::process::Command::new(sidecar_path)
            .env("QDRANT__SERVICE__HTTP_PORT", http_port.to_string())
            .env("QDRANT__SERVICE__GRPC_PORT", grpc_port.to_string())
            .args(["--config-path", config_path.to_str().expect("invalid path")])
            .stdout(std::process::Stdio::piped())
            .spawn()?;

        let pid = Pid::from_child(&process);

        if let Some(stdout) = process.stdout {
            let reader = std::io::BufReader::new(stdout);
            tokio::spawn(async move {
                let mut lines = reader.lines();
                while let Some(line) = lines.next() {
                    if let Ok(line) = line {
                        tracing::debug!("[qdrant bin] {}", line);
                    }
                }
            });
        }

        let url = format!("http://{}:{}", Ipv4Addr::LOCALHOST, grpc_port);

        let liveness =
            check_liveness(SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), http_port)).await;
        if !liveness {
            bail!("qdrant start timeout");
        }

        let client = QdrantClient::from_url(&url).build()?;

        Ok(Self {
            client: Arc::new(client),
            pid,
        })
    }

    pub fn get_client(&self) -> Arc<QdrantClient> {
        self.client.clone()
    }

    pub fn get_pid(&self) -> NonZeroI32 {
        self.pid.as_raw_nonzero()
    }
}

/// Check if qdrant is alive
async fn check_liveness(addr: SocketAddr) -> bool {
    let probe = format!("http://{}:{}", addr.ip(), addr.port());

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
            false
        }
        _ = rx1 => {
            info!("qdrant started");
            true
        }
    }
}

/// Kill qdrant server using pid, if the executable path of pid is qdrant.
///
/// This function will block until the process is killed.
pub fn kill(pid: i32) -> anyhow::Result<()> {
    let pidpath =
        libproc::proc_pid::pidpath(pid).map_err(|e| anyhow!("failed to get pidpath: {}", e))?;
    let current_exe_path = std::env::current_exe().expect("failed to get current executable");
    let sidecar_path = current_exe_path.with_file_name("qdrant");

    if pidpath == sidecar_path.to_string_lossy().to_string() {
        info!("pid {} is qdrant started using sidecar, killing it", pid);
        let pid = Pid::from_raw(pid).ok_or(anyhow!("invalid pid"))?;
        if let Err(e) = kill_process(pid, Signal::Term) {
            bail!("failed to send SIGTERM: {}", e);
        }

        // waiting the process to exit
        let (tx, rx) = channel();
        std::thread::spawn(move || loop {
            if libproc::proc_pid::pidpath(pid.as_raw_nonzero().into()).is_err() {
                let _ = tx.send(());
            }
            sleep(std::time::Duration::from_millis(100));
        });

        rx.recv()
            .map_err(|e| anyhow!("failed to kill qdrant: {}", e))
    } else {
        warn!("pid {} is not qdrant started using sidecar, ignore it", pid);
        Ok(())
    }
}

/// Kill qdrant server async using pid, if the executable path of pid is qdrant.
///
/// When this function returns, the process has been killed.
pub async fn kill_async(pid: i32) -> anyhow::Result<()> {
    let pidpath =
        libproc::proc_pid::pidpath(pid).map_err(|e| anyhow!("failed to get pidpath: {}", e))?;
    let current_exe_path = std::env::current_exe().expect("failed to get current executable");
    let sidecar_path = current_exe_path.with_file_name("qdrant");

    if pidpath == sidecar_path.to_string_lossy().to_string() {
        info!("pid {} is qdrant started using sidecar, killing it", pid);
        let pid = Pid::from_raw(pid).ok_or(anyhow!("invalid pid"))?;
        if let Err(e) = kill_process(pid, Signal::Term) {
            bail!("failed to send SIGTERM: {}", e);
        }

        // waiting the process to exit
        let (tx, rx) = oneshot::channel();
        tokio::spawn(async move {
            if libproc::proc_pid::pidpath(pid.as_raw_nonzero().into()).is_err() {
                let _ = tx.send(());
            }
            sleep(std::time::Duration::from_millis(100));
        });

        rx.await
            .map_err(|e| anyhow!("failed to kill qdrant: {}", e))
    } else {
        warn!("pid {} is not qdrant started using sidecar, ignore it", pid);
        Ok(())
    }
}

impl Drop for QdrantServer {
    fn drop(&mut self) {
        info!("qdrant server dropped");
        match kill_process(self.pid, Signal::Term) {
            Ok(_) => {
                info!("qdrant successfully killed");
            }
            Err(e) => {
                error!("failed to kill qdrant: {}", e);
            }
        }
    }
}
