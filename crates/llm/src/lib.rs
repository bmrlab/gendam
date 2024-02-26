pub mod model;

use anyhow::bail;
use llama_cpp::{standard_sampler::StandardSampler, LlamaModel, LlamaParams, SessionParams};
use std::path::Path;
use tokio::sync::{
    mpsc::{self, Sender},
    oneshot,
};
use tracing::{debug, error};

pub enum LLMResult {
    Text(String),
    Error,
}

pub struct LLMInput {
    prompt: String,
    tx: oneshot::Sender<LLMResult>,
}

pub enum LLMPayload {
    Input(LLMInput),
    Stop,
}

pub struct LLM {
    tx: Sender<LLMPayload>,
}

impl LLM {
    pub async fn new(
        resources_dir: impl AsRef<Path>,
        model: self::model::Model,
    ) -> anyhow::Result<Self> {
        let model_uri = match model {
            self::model::Model::Gemma2B => "Gemma/2b.gguf",
            self::model::Model::QWen0_5B => "qwen/0.5b.gguf",
        };

        let download = file_downloader::FileDownload::new(file_downloader::FileDownloadConfig {
            resources_dir: resources_dir.as_ref().to_path_buf(),
            ..Default::default()
        });

        let model_path = download.download_if_not_exists(model_uri).await?;

        // FIXME GPU 相关参数需要进一步调整
        let params = LlamaParams {
            n_gpu_layers: 0, // disable gpu for now
            main_gpu: 0,
            ..Default::default()
        };
        let model = LlamaModel::load_from_file_async(model_path, params)
            .await
            .unwrap();

        debug!("model loaded");

        let (tx, mut rx) = mpsc::channel::<LLMPayload>(512);

        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;

        std::thread::spawn(move || {
            let local = tokio::task::LocalSet::new();

            local.spawn_local(async move {
                loop {
                    while let Some(payload) = rx.recv().await {
                        match payload {
                            LLMPayload::Input(input) => {
                                debug!("input received");

                                let mut params = SessionParams::default();
                                params.n_batch = 512;
                                params.n_ctx = 512;

                                let mut ctx = model.create_session(params).unwrap();

                                // ctx.advance_context("<SYSTEM>you are a helpful assistant").unwrap();
                                // ctx.advance_context("<USER>who are you?").unwrap();
                                let tokens = ctx
                                    .model()
                                    .tokenize_bytes(input.prompt, true, true)
                                    .unwrap();
                                ctx.advance_context_with_tokens(tokens).unwrap();

                                let mut results = vec![];

                                let mut completions =
                                    ctx.start_completing_with(StandardSampler::default(), 1024);

                                while let Some(token) = completions.next_token_async().await {
                                    let formatted = model.token_to_piece(token);
                                    tracing::info!("{:?}", formatted);
                                    results.push(formatted);

                                    if token == model.eos() {
                                        break;
                                    }
                                }

                                tracing::debug!("{:?}", results.join(""));

                                let _ = input.tx.send(LLMResult::Text(results.join("")));
                            }
                            _ => {
                                todo!()
                            }
                        }
                    }
                }
            });

            rt.block_on(local);
        });

        Ok(Self { tx })
    }

    pub async fn call(&self, prompt: &str) -> anyhow::Result<String> {
        let (tx, rx) = oneshot::channel::<LLMResult>();

        debug!("start send input");

        self.tx
            .send(LLMPayload::Input(LLMInput {
                prompt: prompt.into(),
                tx,
            }))
            .await?;

        debug!("finished");

        match rx.await {
            Ok(LLMResult::Text(text)) => Ok(text),
            Ok(LLMResult::Error) => bail!("error from LLM"),
            Err(e) => {
                error!("channel error");
                Err(e.into())
            }
        }
    }
}

#[test_log::test(tokio::test)]
async fn test_llm() {
    let resources_dir = "/Users/zhuo/Library/Application Support/cc.musedam.local/resources";
    let llm = LLM::new(resources_dir, self::model::Model::Gemma2B)
        .await
        .unwrap();

    let response = llm.call("who are you?").await;

    println!("{:?}", response);

    assert!(response.is_ok());
}
