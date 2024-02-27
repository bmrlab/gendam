pub mod model;

use anyhow::bail;
use llama_cpp::{LlamaModel, LlamaParams};
use std::path::Path;
use tokio::sync::{
    mpsc::{self, Sender},
    oneshot,
};
use tracing::{debug, error};

pub use llama_cpp::{standard_sampler::StandardSampler, SessionParams};

pub enum LLMResult {
    Text(String),
    Error,
}

#[derive(Debug)]
pub enum LLMMessage {
    System(String),
    User(String),
    Assistant(String),
}

pub struct LLMInput {
    history: Vec<LLMMessage>,
    session_params: Option<SessionParams>,
    sampler: Option<StandardSampler>,
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
        let model_uri = model.model_uri();
        let download = file_downloader::FileDownload::new(file_downloader::FileDownloadConfig {
            resources_dir: resources_dir.as_ref().to_path_buf(),
            ..Default::default()
        });
        let input_model = model.clone();

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
                                let params = {
                                    match input.session_params {
                                        Some(params) => params,
                                        _ => SessionParams::default(),
                                    }
                                };

                                let mut ctx = model.create_session(params).unwrap();

                                let prompt = input_model.with_chat_template(input.history);

                                debug!("final prompt: {}", prompt);

                                let tokens =
                                    ctx.model().tokenize_bytes(prompt, true, true).unwrap();
                                ctx.advance_context_with_tokens(tokens).unwrap();

                                let mut results = vec![];

                                let sampler = match input.sampler {
                                    Some(sampler) => sampler,
                                    _ => StandardSampler::default(),
                                };

                                let mut completions = ctx.start_completing_with(sampler, 1024);

                                while let Some(token) = completions.next_token_async().await {
                                    let formatted = model.token_to_piece(token);
                                    results.push(formatted);

                                    if token == model.eos() {
                                        break;
                                    }
                                }

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

    pub async fn call(
        &self,
        history: Vec<LLMMessage>,
        session_params: Option<SessionParams>,
        sampler: Option<StandardSampler>,
    ) -> anyhow::Result<String> {
        let (tx, rx) = oneshot::channel::<LLMResult>();

        self.tx
            .send(LLMPayload::Input(LLMInput {
                history,
                tx,
                session_params,
                sampler,
            }))
            .await?;

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
    let llm = LLM::new(resources_dir, self::model::Model::QWen0_5B)
        .await
        .unwrap();

    let prompt = r#"You are an AI assistant designed for summarizing a video.
Following document records what people see and hear from a video.
Please summarize the video content in one sentence based on the document.
The sentence should not exceed 30 words.
If you cannot summarize, just response with empty message.
Please start with "The video contains".
Do not repeat the information in document.
Do not response any other information.

Here is the document:

a close up of a cell phone with pictures of people on it
a close up of a cell phone with pictures of people on it
a close up of a cell phone with pictures of people on it
a close up of a cell phone with pictures of people on it
a close up of a cell phone with pictures of people on it"#;

    let response = llm
        .call(
            vec![LLMMessage::User(prompt.into())],
            None,
            Some(StandardSampler {
                temp: 0.0,
                ..Default::default()
            }),
        )
        .await;

    println!("{:?}", response);

    assert!(response.is_ok());
}
