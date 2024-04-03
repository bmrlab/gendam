use crate::{model::LlamaCppModel, Chat, Embedding, LLMImageContent, LLMMessage, LLMParams};
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
use tracing::{debug, warn};

pub struct NativeModel {
    model: LlamaModel,
    backend: LlamaBackend,
    input_model: LlamaCppModel,
    is_multimodal: bool,
}

#[async_trait]
impl Chat for NativeModel {
    async fn get_completion(
        &self,
        history: Vec<LLMMessage>,
        _images: Option<Vec<LLMImageContent>>,
        params: Option<LLMParams>,
    ) -> anyhow::Result<String> {
        let prompt = self.input_model.with_chat_template(history);
        get_completion(prompt, params.map(|v| v.into()), &self.model, &self.backend)
    }

    fn is_multimodal_chat(&self) -> bool {
        self.is_multimodal
    }
}

#[async_trait]
impl Embedding for NativeModel {
    async fn get_embedding(
        &self,
        prompt: String,
        _images: Option<Vec<LLMImageContent>>,
        params: Option<LLMParams>,
    ) -> anyhow::Result<Vec<f32>> {
        get_embedding(prompt, params.map(|v| v.into()), &self.model, &self.backend)
    }

    fn is_multimodal_embedding(&self) -> bool {
        self.is_multimodal
    }
}

impl NativeModel {
    pub async fn new(
        resources_dir: impl AsRef<Path>,
        model: LlamaCppModel,
    ) -> anyhow::Result<impl Chat + Embedding> {
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

        Ok(Self {
            model,
            backend,
            input_model,
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

fn get_embedding(
    prompt: String,
    ctx_params: Option<LlamaContextParams>,
    model: &LlamaModel,
    backend: &LlamaBackend,
) -> anyhow::Result<Vec<f32>> {
    debug!("start embedding");
    let ctx_params = match ctx_params {
        Some(params) => params,
        // TODO not sure if it is ok to use n_ctx_train as default n_ctx and n_batch
        None => LlamaContextParams::default()
            .with_n_ctx(NonZeroU32::new(model.n_ctx_train() as u32))
            .with_n_batch(model.n_ctx_train() as u32),
    };

    match model.new_context(&backend, ctx_params.with_embeddings(true)) {
        Ok(mut ctx) => {
            debug!("context created");

            let tokens_list = model
                .str_to_token(&prompt, AddBos::Always)
                .expect("failed to tokenize prompt");

            let mut batch = LlamaBatch::new(model.n_ctx_train() as usize, 1);
            batch.add_sequence(&tokens_list, 0, false)?;
            ctx.decode(&mut batch)?;
            let embedding = ctx.embeddings_seq_ith(0)?;
            let magnitude = embedding
                .iter()
                .fold(0.0, |acc, &val| val.mul_add(val, acc))
                .sqrt();

            let embedding = embedding
                .iter()
                .map(|&val| val / magnitude)
                .collect::<Vec<_>>();

            Ok(embedding)
        }
        Err(e) => {
            bail!("failed to create context: {}", e);
        }
    }
}

#[test_log::test(tokio::test)]
async fn test_native_llm() {
    let model = NativeModel::new(
        "/Users/zhuo/dev/tezign/bmrlab/tauri-dam-test-playground/apps/desktop/src-tauri/resources",
        LlamaCppModel::QWen0_5B,
    )
    .await
    .expect("failed to create model");

    let response = model
        .get_completion(vec![LLMMessage::User("who are you?".into())], None, None)
        .await;

    tracing::info!("response: {:?}", response);

    let response = model
        .get_completion(
            vec![LLMMessage::User(
                "can you write a simple quick sort in python?".into(),
            )],
            None,
            None,
        )
        .await;

    tracing::info!("response: {:?}", response);
}

#[test_log::test(tokio::test)]
async fn test_native_llm_embedding() {
    let model = NativeModel::new(
        "/Users/zhuo/dev/tezign/bmrlab/tauri-dam-test-playground/apps/desktop/src-tauri/resources",
        LlamaCppModel::Gemma2B,
    )
    .await
    .expect("failed to create model");

    let embedding = model.get_embedding("who are you?".into(), None, None).await;

    tracing::info!("embedding: {:?}", embedding);
}
