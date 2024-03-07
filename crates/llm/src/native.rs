use crate::{
    model::LlamaCppModel, Chat, LLMImageContent, LLMInput, LLMMessage, LLMParams, LLMPayload,
};
use anyhow::bail;
use async_trait::async_trait;
use llama_cpp_2::{
    context::params::LlamaContextParams,
    llama_backend::LlamaBackend,
    llama_batch::LlamaBatch,
    model::{params::LlamaModelParams, AddBos, LlamaModel},
    token::data_array::LlamaTokenDataArray,
};
use std::{num::NonZeroU32, path::Path};
use tokio::sync::{
    mpsc::{self, Sender},
    oneshot,
};
use tracing::{debug, error, warn};

pub struct NativeModel {
    tx: Sender<LLMPayload>,
    is_multimodal: bool,
}

#[async_trait]
impl Chat for NativeModel {
    async fn get_completion(
        &self,
        history: Vec<LLMMessage>,
        images: Option<Vec<LLMImageContent>>,
        params: Option<LLMParams>,
    ) -> anyhow::Result<String> {
        let (tx, rx) = oneshot::channel::<anyhow::Result<String>>();

        self.tx
            .send(LLMPayload::Input(LLMInput {
                history,
                tx,
                params,
                images: images.unwrap_or(vec![]),
            }))
            .await?;

        match rx.await {
            Ok(Ok(text)) => Ok(text),
            Ok(Err(e)) => bail!("error from LLM: {}", e),
            Err(e) => {
                error!("channel error");
                Err(e.into())
            }
        }
    }

    fn is_multimodal(&self) -> bool {
        self.is_multimodal
    }
}

impl NativeModel {
    pub async fn new(
        resources_dir: impl AsRef<Path>,
        model: LlamaCppModel,
    ) -> anyhow::Result<impl Chat> {
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

        // start a new local thread where model will run, and make sure model will never be moved
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
                                let prompt = input_model.with_chat_template(input.history);
                                debug!("final prompt: {}", prompt);

                                match get_completion(
                                    prompt,
                                    input.params.map(|v| v.into()),
                                    &model,
                                    &backend,
                                ) {
                                    Ok(result) => {
                                        let _ = input.tx.send(Ok(result));
                                    }
                                    Err(e) => {
                                        let _ = input.tx.send(Err(e));
                                    }
                                }
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

        Ok(Self {
            tx,
            // native model do not support image input for now
            is_multimodal: false,
        })
    }
}

fn get_completion(
    prompt: String,
    ctx_params: Option<LlamaContextParams>,
    model: &LlamaModel,
    backend: &LlamaBackend,
) -> anyhow::Result<String> {
    debug!("start get_completion");
    let ctx_params = match ctx_params {
        Some(params) => params,
        // TODO not sure if it is ok to use n_ctx_train as default n_ctx and n_batch
        None => LlamaContextParams::default()
            .with_n_ctx(NonZeroU32::new(model.n_ctx_train() as u32))
            .with_n_batch(model.n_ctx_train() as u32),
    };

    match model.new_context(&backend, ctx_params) {
        Ok(mut ctx) => {
            debug!("context created");

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
                if let Err(e) = batch.add(token, i, &[0], is_last) {
                    warn!("failed to add token: {}", e);
                }
            }

            if let Err(e) = ctx.decode(&mut batch) {
                warn!("failed to decode tokens: {}", e);
            }

            let mut n_cur = batch.n_tokens();

            while n_cur <= n_len {
                {
                    let candidates = ctx.candidates_ith(batch.n_tokens() - 1);

                    let candidates_p = LlamaTokenDataArray::from_iter(candidates, false);

                    // sample the most likely token
                    let new_token_id = ctx.sample_token_greedy(candidates_p);

                    // is it an end of stream?
                    if new_token_id == model.token_eos() {
                        break;
                    }

                    match model.token_to_str(new_token_id) {
                        Ok(token) => {
                            results.push(token);
                        }
                        _ => {
                            warn!("failed to convert token to str: {}", new_token_id);
                        }
                    };

                    batch.clear();
                    if let Err(e) = batch.add(new_token_id, n_cur, &[0], true) {
                        warn!("failed to add token: {}", e)
                    }
                }
                n_cur += 1;

                if let Err(e) = ctx.decode(&mut batch) {
                    warn!("failed to decode batch: {}", e);
                }
            }

            Ok(results.join(""))
        }
        Err(e) => {
            bail!("failed to create context: {}", e);
        }
    }
}
