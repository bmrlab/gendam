pub mod model;

use anyhow::bail;
pub use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::AddBos;
use llama_cpp_2::model::LlamaModel;
use llama_cpp_2::token::data_array::LlamaTokenDataArray;
use std::num::NonZeroU32;
use std::path::Path;
use tokio::sync::{
    mpsc::{self, Sender},
    oneshot,
};
use tracing::warn;
use tracing::{debug, error};

#[derive(Debug)]
pub enum LLMMessage {
    System(String),
    User(String),
    Assistant(String),
}

pub enum LLMResult {
    Text(String),
    Error,
}

pub struct LLMInput {
    history: Vec<LLMMessage>,
    ctx_params: Option<LlamaContextParams>,
    // sampler: Option<StandardSampler>,
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
        // init backend
        let backend = LlamaBackend::init()?;
        // offload all layers to the gpu
        let model_params = { LlamaModelParams::default() };

        let model_uri = model.model_uri();
        let download = file_downloader::FileDownload::new(file_downloader::FileDownloadConfig {
            resources_dir: resources_dir.as_ref().to_path_buf(),
            ..Default::default()
        });
        let input_model = model.clone();

        let model_path = download.download_if_not_exists(model_uri).await?;

        let model = LlamaModel::load_from_file(&backend, model_path, &model_params)?;

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
                                let ctx_params = match input.ctx_params {
                                    Some(params) => params,
                                    _ => LlamaContextParams::default(),
                                }
                                .with_n_ctx(NonZeroU32::new(model.n_ctx_train() as u32))
                                .with_n_batch(model.n_ctx_train() as u32);

                                let mut ctx = model
                                    .new_context(&backend, ctx_params)
                                    .expect("failed to create context");

                                let prompt = input_model.with_chat_template(input.history);

                                debug!("final prompt: {}", prompt);

                                let tokens_list = model
                                    .str_to_token(&prompt, AddBos::Always)
                                    .expect("failed to tokenize prompt");

                                let n_len = model.n_ctx_train() as i32;

                                let mut results = vec![];

                                let mut batch = LlamaBatch::new(model.n_ctx_train() as usize, 1);

                                let last_index: i32 = (tokens_list.len() - 1) as i32;
                                for (i, token) in (0_i32..).zip(tokens_list.into_iter()) {
                                    // llama_decode will output logits only for the last token of the prompt
                                    let is_last = i == last_index;
                                    batch
                                        .add(token, i, &[0], is_last)
                                        .expect("failed to add token");
                                }

                                ctx.decode(&mut batch).expect("failed to decode tokens");

                                let mut n_cur = batch.n_tokens();

                                while n_cur <= n_len {
                                    {
                                        let candidates = ctx.candidates_ith(batch.n_tokens() - 1);

                                        let candidates_p =
                                            LlamaTokenDataArray::from_iter(candidates, false);

                                        // sample the most likely token
                                        let new_token_id = ctx.sample_token_greedy(candidates_p);

                                        // is it an end of stream?
                                        if new_token_id == model.token_eos() {
                                            eprintln!();
                                            break;
                                        }

                                        match model.token_to_str(new_token_id) {
                                            Ok(token) => {
                                                results.push(token);
                                            }
                                            _ => {
                                                warn!(
                                                    "failed to convert token to str: {}",
                                                    new_token_id
                                                );
                                            }
                                        };

                                        batch.clear();
                                        batch
                                            .add(new_token_id, n_cur, &[0], true)
                                            .expect("failed to add token");
                                    }
                                    n_cur += 1;

                                    ctx.decode(&mut batch).expect("failed to decode tokens");
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
        ctx_params: Option<LlamaContextParams>,
    ) -> anyhow::Result<String> {
        let (tx, rx) = oneshot::channel::<LLMResult>();

        self.tx
            .send(LLMPayload::Input(LLMInput {
                history,
                tx,
                ctx_params,
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

    let response = llm.call(vec![LLMMessage::User(prompt.into())], None).await;

    println!("{:?}", response);

    assert!(response.is_ok());
}
