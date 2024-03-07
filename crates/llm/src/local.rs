use crate::{Chat, LLMImageContent, LLMMessage, LLMParams};
use anyhow::bail;
use async_trait::async_trait;
use reqwest::{self, Url};
use serde::{Deserialize, Serialize};
use std::{fs, os::unix::fs::PermissionsExt, str::FromStr};
use tokio::sync::oneshot;
use tracing::{debug, error, info};

// it seems that there is not more options
enum ServerAction {
    Kill,
}

#[allow(dead_code)]
pub struct LocalModel {
    api_endpoint: Url,
    api_secret: Option<String>,
    server_tx: std::sync::mpsc::Sender<ServerAction>,
    model: crate::model::LlamaCppModel,
    is_multimodal: bool,
}

#[derive(Serialize, Deserialize)]
struct CompletionPayload {
    prompt: String,
    seed: Option<u32>,
    temperature: Option<f32>,
    image_data: Option<Vec<LLMImageContent>>,
}

#[derive(Serialize, Deserialize)]
struct CompletionResponse {
    content: String,
}

impl CompletionPayload {
    pub fn from(
        prompt: String,
        images: Option<Vec<LLMImageContent>>,
        params: Option<LLMParams>,
    ) -> Self {
        // FIXME find some way to simplify the code
        Self {
            prompt,
            seed: match &params {
                Some(params) => params.seed,
                None => None,
            },
            temperature: match &params {
                Some(params) => params.temperature,
                None => None,
            },
            image_data: images,
        }
    }
}

#[async_trait]
impl Chat for LocalModel {
    async fn get_completion(
        &self,
        history: Vec<LLMMessage>,
        images: Option<Vec<LLMImageContent>>,
        params: Option<LLMParams>,
    ) -> anyhow::Result<String> {
        let prompt = self.model.with_chat_template(history);
        let payload = CompletionPayload::from(prompt, images, params);

        let client = reqwest::Client::new();
        let resp = client
            .post(self.api_endpoint.join("/completion")?)
            .body(serde_json::to_string(&payload)?)
            .send()
            .await?;

        if resp.status() != reqwest::StatusCode::OK {
            bail!("error from LLM server: {}", resp.text().await?);
        }

        let resp = resp.json::<CompletionResponse>().await?;

        Ok(resp.content)
    }

    fn is_multimodal(&self) -> bool {
        self.is_multimodal
    }
}

impl LocalModel {
    pub async fn new(
        resources_dir: impl AsRef<std::path::Path>,
        model: crate::model::LlamaCppModel,
    ) -> anyhow::Result<impl Chat> {
        let download = file_downloader::FileDownload::new(file_downloader::FileDownloadConfig {
            resources_dir: resources_dir.as_ref().to_path_buf(),
            ..Default::default()
        });

        let binary_path = download.download_if_not_exists("llama/server").await?;
        download
            .download_if_not_exists("llama/ggml-metal.metal")
            .await?;

        // set binary permission to executable
        let mut perms = fs::metadata(&binary_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&binary_path, perms)?;

        let mut args: Vec<String> = vec!["-m".into()];

        let model_path = download.download_if_not_exists(model.model_uri()).await?;
        args.push(model_path.to_str().unwrap().into());

        match model.mmproj_uri() {
            Some(uri) => {
                let path = download.download_if_not_exists(uri).await?;

                args.push("--mmproj".into());
                args.push(path.to_str().unwrap().into());
            }
            _ => {}
        };

        // TODO maybe we should use a random port
        let api_endpoint = Url::from_str("http://localhost:8080")?;

        match api_endpoint.port() {
            Some(port) => {
                args.extend(["--port".into(), port.to_string()]);
            }
            None => {
                bail!("no port available in api endpoint: {}", api_endpoint);
            }
        }

        let (tx, rx) = std::sync::mpsc::channel::<ServerAction>();

        // run llama cpp server
        match std::process::Command::new(binary_path).args(args).spawn() {
            Ok(mut process) => {
                std::thread::spawn(move || loop {
                    if let Ok(action) = rx.recv() {
                        match action {
                            ServerAction::Kill => {
                                process.kill().unwrap();
                            }
                        }
                    }
                });
            }
            Err(e) => {
                bail!("failed to spawn llama cpp server: {}", e);
            }
        }

        // use channel and select to make sure server is started
        let (tx1, rx1) = oneshot::channel();
        let endpoint = api_endpoint.clone();
        tokio::spawn(async move {
            loop {
                let resp = reqwest::get(endpoint.join("/health").expect("invalid url")).await;
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
                bail!("llama cpp server start timeout");
            }
            _ = rx1 => {
                debug!("llama cpp server started");
            }
        }

        debug!("success!");

        Ok(Self {
            api_endpoint,
            api_secret: None,
            is_multimodal: model.is_multi_modal(),
            model,
            server_tx: tx,
        })
    }
}

impl Drop for LocalModel {
    fn drop(&mut self) {
        match self.server_tx.send(ServerAction::Kill) {
            Ok(_) => {
                info!("llama cpp server successfully killed");
            }
            Err(e) => {
                error!("failed to kill llama cpp server: {}", e);
            }
        }
    }
}
