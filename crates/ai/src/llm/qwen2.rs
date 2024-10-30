use crate::LLMOutput;

use super::{
    candle::CandleLLMModel, native::LocalLLMModel, LLMInferenceParams, LLMMessage, LLMModel,
};
use anyhow::bail;
use candle_core::{quantized::gguf_file, Device, Tensor};
use candle_transformers::models::quantized_qwen2;
use std::path::Path;
use tokenizers::{tokenizer, Tokenizer};
use tokio::sync::mpsc;

#[derive(Clone)]
pub struct Qwen2 {
    tx: std::sync::mpsc::Sender<LLMInferencePayload>,
    tokenizer: Tokenizer,
    device: Device,
}

#[derive(Clone, Debug)]
struct LLMInferenceForwardPayload {
    input: Tensor,
    index_pos: usize,
    tx: std::sync::mpsc::Sender<anyhow::Result<Tensor>>,
}

#[derive(Clone, Debug)]
enum LLMInferencePayload {
    Completion(LLMInferenceForwardPayload),
}

impl LLMModel for Qwen2 {
    async fn get_completion(
        &self,
        history: &[LLMMessage],
        params: LLMInferenceParams,
    ) -> anyhow::Result<LLMOutput> {
        let prompt = self.with_chat_template(history);

        let (tx, mut rx) = mpsc::channel(512);
        let self_clone = self.clone();
        tokio::spawn(async move {
            if let Err(e) = self_clone.forward(&prompt, params, Some(tx)).await {
                tracing::error!("qwen2 forward error: {}", e);
            }
        });

        let stream = async_stream::stream! {
            while let Some(v) = rx.recv().await {
                yield v;
            }
        };

        Ok(LLMOutput::new(Box::pin(stream)))
    }
}

impl LocalLLMModel for Qwen2 {
    fn start_of_turn(&self) -> String {
        "<|im_start|>".to_string()
    }

    fn end_of_turn(&self) -> String {
        "<|im_end|>".to_string()
    }

    fn system_name(&self) -> String {
        "system".to_string()
    }

    fn user_name(&self) -> String {
        "user".to_string()
    }

    fn assistant_name(&self) -> String {
        "assistant".to_string()
    }
}

impl CandleLLMModel for Qwen2 {
    fn next_token_logits(
        &self,
        input: &candle_core::Tensor,
        index_pos: usize,
    ) -> anyhow::Result<candle_core::Tensor> {
        let (tx, rx) = std::sync::mpsc::channel();

        let _ = self.tx.send(LLMInferencePayload::Completion(
            LLMInferenceForwardPayload {
                input: input.clone(),
                index_pos,
                tx,
            },
        ));

        match rx.recv() {
            Ok(result) => result,
            _ => bail!("failed to receive result"),
        }
    }

    fn tokenizers(&self) -> tokenizer::Tokenizer {
        self.tokenizer.clone()
    }

    fn device(&self) -> candle_core::Device {
        self.device.clone()
    }
}

impl Qwen2 {
    pub fn load(
        model_path: impl AsRef<Path>,
        tokenizer_path: impl AsRef<Path>,
        device: &str,
    ) -> anyhow::Result<Self> {
        let device = match device {
            "metal" => match Device::new_metal(0) {
                Ok(v) => v,
                _ => Device::Cpu,
            },
            "cuda" => {
                bail!("cuda device is not supported for now");
            }
            _ => Device::Cpu,
        };

        let tokenizer = Tokenizer::from_file(tokenizer_path).map_err(|e| anyhow::anyhow!(e))?;
        let mut file = std::fs::File::open(&model_path)?;
        let model = gguf_file::Content::read(&mut file).map_err(|e| e.with_path(model_path))?;
        let mut model = quantized_qwen2::ModelWeights::from_gguf(model, &mut file, &device)?;

        let (tx, rx) = std::sync::mpsc::channel();

        std::thread::spawn(move || {
            loop {
                match rx.recv() {
                    Ok(payload) => {
                        match payload {
                            LLMInferencePayload::Completion(payload) => {
                                // let _ = tx.send(self.forward_completion(payload).ok());
                                let result = model
                                    .forward(&payload.input, payload.index_pos)
                                    .map_err(|err| anyhow::anyhow!(err));
                                if let Err(e) = payload.tx.send(result) {
                                    tracing::error!("qwen2 completion error: {:?}", e);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        tracing::warn!("qwen2 rx error: {:?}", e);
                        break;
                    }
                }
            }
        });

        Ok(Self {
            tx,
            tokenizer,
            device: device.to_owned(),
        })
    }
}
