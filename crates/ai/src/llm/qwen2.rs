use super::{
    candle::CandleLLMModel, native::LocalLLMModel, LLMInferenceParams, LLMMessage, LLMModel,
};
use anyhow::bail;
use candle_core::{quantized::gguf_file, Device};
use candle_transformers::models::quantized_qwen2;
use std::path::Path;
use tokenizers::{tokenizer, Tokenizer};

pub struct Qwen2 {
    tokenizer: tokenizer::Tokenizer,
    model: quantized_qwen2::ModelWeights,
    device: Device,
}

impl LLMModel for Qwen2 {
    async fn get_completion(
        &mut self,
        history: &[LLMMessage],
        params: LLMInferenceParams,
    ) -> anyhow::Result<String> {
        let prompt = self.with_chat_template(history);
        self.forward(&prompt, params)
    }

    async fn get_completion_with_image(
        &mut self,
        _history: &[super::LLMMessage],
        _image_url: &str,
        _params: super::LLMInferenceParams,
    ) -> anyhow::Result<String> {
        bail!("Qwen2 does not support chat with image input")
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
        &mut self,
        input: &candle_core::Tensor,
        index_pos: usize,
    ) -> anyhow::Result<candle_core::Tensor> {
        self.model.forward(input, index_pos).map_err(|e| e.into())
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
        device: Device,
    ) -> anyhow::Result<Self> {
        let tokenizer = Tokenizer::from_file(tokenizer_path).map_err(|e| anyhow::anyhow!(e))?;
        let mut file = std::fs::File::open(&model_path)?;
        let model = gguf_file::Content::read(&mut file).map_err(|e| e.with_path(model_path))?;
        let model = quantized_qwen2::ModelWeights::from_gguf(model, &mut file, &device)?;

        Ok(Self {
            tokenizer,
            model,
            device: device.to_owned(),
        })
    }
}
