mod quantized_llava_phi3;
use crate::{
    llm::candle::TokenOutputStream,
    traits::{ImageCaptionInput, ImageCaptionOutput, Model},
};
use candle_core::{Device, IndexOp, Tensor};
use candle_transformers::generation::{LogitsProcessor, Sampling};
use quantized_llava_phi3::{format_prompt, HFPreProcessorConfig, ImageProcessor, QLLaVAPhi3};
use std::path::Path;

/// The length of the sample to generate (in tokens).
const SAMPLE_LEN: usize = 100; // A limit instruction exists in the prompt
const REPEAT_PENALTY: f32 = 1.1;
const REPEAT_LAST_N: usize = 64;

impl Model for LLaVAPhi3Mini {
    type Item = ImageCaptionInput;
    type Output = ImageCaptionOutput;

    fn batch_size_limit(&self) -> usize {
        1
    }

    async fn process(
        &mut self,
        items: Vec<Self::Item>,
    ) -> anyhow::Result<Vec<anyhow::Result<Self::Output>>> {
        if items.len() > self.batch_size_limit() {
            anyhow::bail!("too many items");
        }
        let mut results: Vec<Result<String, anyhow::Error>> = vec![];
        for item in items {
            let res = self.get_image_caption(item);
            results.push(res);
        }
        Ok(results)
    }
}

#[derive(Debug)]
pub struct LLaVAPhi3Mini {
    device: Device,
    image_processor: ImageProcessor,
    inner: QLLaVAPhi3,
}

impl LLaVAPhi3Mini {
    pub fn new(
        device: &str,
        gguf_model_path: impl AsRef<Path>,
        mmproj_gguf_model_path: impl AsRef<Path>,
        tokenizer_path: impl AsRef<Path>,
        preprocessor_config_path: impl AsRef<Path>,
    ) -> anyhow::Result<Self> {
        let device = match device {
            "metal" => Device::new_metal(0)?,
            _ => anyhow::bail!("Unsupported device: {}", device),
        };
        let preprocessor_config: HFPreProcessorConfig =
            serde_json::from_slice(&std::fs::read(preprocessor_config_path)?)?;
        let image_processor = ImageProcessor::from_hf_preprocessor_config(&preprocessor_config);
        let qllavaphi3 = QLLaVAPhi3::load(
            &device,
            gguf_model_path,
            mmproj_gguf_model_path,
            tokenizer_path,
        )?;
        Ok(Self {
            device,
            image_processor,
            inner: qllavaphi3,
        })
    }

    fn generate(
        &mut self,
        mut input_embeds: Tensor,
        seed: u64,
        temperature: f64,
    ) -> anyhow::Result<String> {
        let QLLaVAPhi3 {
            llama,
            tokenizer,
            config,
            ..
        } = &mut self.inner;
        let mut tos = TokenOutputStream::new(tokenizer.clone());
        let sampling = {
            if temperature <= 0. {
                Sampling::ArgMax
            } else {
                Sampling::All { temperature }
            }
        };
        let mut logits_processor = LogitsProcessor::from_sampling(seed, sampling.clone());
        let mut index_pos = 0;
        let mut all_tokens = vec![];
        for index in 0..SAMPLE_LEN.saturating_sub(1) {
            let (_, input_embeds_len, _) = input_embeds.dims3()?;
            // use kv cache, it is implemented in quantized llama
            let (context_size, context_index) = if index > 0 {
                (1, index_pos)
            } else {
                (input_embeds_len, 0)
            };
            let input =
                input_embeds.i((.., input_embeds_len.saturating_sub(context_size).., ..))?;
            let logits = llama.forward_input_embed(&input, context_index)?;
            let logits = logits.squeeze(0)?;

            let logits = if REPEAT_PENALTY == 1. {
                logits
            } else {
                let start_at = all_tokens.len().saturating_sub(REPEAT_LAST_N);
                candle_transformers::utils::apply_repeat_penalty(
                    &logits,
                    REPEAT_PENALTY,
                    &all_tokens[start_at..],
                )?
            };

            let (_, input_len, _) = input.dims3()?;
            index_pos += input_len;
            let next_token = logits_processor.sample(&logits)?;
            all_tokens.push(next_token);
            let next_token_tensor = Tensor::from_vec(vec![next_token], 1, &self.device)?;
            let next_embeds = llama.embed(&next_token_tensor)?.unsqueeze(0)?;
            input_embeds = Tensor::cat(&[input_embeds, next_embeds], 1)?;
            if next_token == config.eos_token_id {
                break;
            }
            tos.next_token(next_token)?;
        }
        let result = tos.decode_all()?;
        Ok(result)
    }

    #[tracing::instrument(level = "info", skip(self, image_file_path))]
    pub fn get_image_caption(
        &mut self,
        image_file_path: impl AsRef<Path>,
    ) -> anyhow::Result<String> {
        let prompt_str = format_prompt(
            r#"You are an advanced image analysis AI. Examine the image and describe its contents in a concise, text-only format. Focus on identifying: People (including celebrities), actions, objects, animals or pets, nature elements, visual cues of sounds, human speech (if text bubbles present), displayed text (OCR), and brand logos. Provide specific examples for each category found in the image. Only mention categories that are present; omit any that are not detected. Use plain text format without lists or JSON. Be accurate and concise in your descriptions. Limit your response to no more than 50 words."#,
        );

        let img = image::ImageReader::open(image_file_path)?
            .with_guessed_format()?
            .decode()?;
        let image_tensor = self.image_processor.preprocess(&img)?.unsqueeze(0)?;
        // let image_tensor = image_tensor.to_dtype(DType::BF16)?.to_device(&device)?;
        let image_tensor = image_tensor.to_device(&self.device)?;
        let image_size = (img.width(), img.height());

        let tokens = self.inner.tokenizer_image_token(prompt_str.as_str())?;

        let input_embeds = self.inner.prepare_inputs_labels_for_multimodal(
            &self.device,
            &tokens,
            &[image_tensor],
            &[image_size],
        )?;

        let caption = self.generate(input_embeds, 299792458, 0.2)?;

        Ok(caption)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    pub async fn test_llava_phi3_mini() -> anyhow::Result<()> {
        let mut llavaphi3mini = LLaVAPhi3Mini::new(
            "metal",
            "/Users/xddotcom/workspace/rust/llm-playground/models/llava-phi-3/llava-phi-3-mini-int4.gguf",
            "/Users/xddotcom/workspace/rust/llm-playground/models/llava-phi-3/llava-phi-3-mini-mmproj-f16.gguf",
            "/Users/xddotcom/workspace/rust/llm-playground/models/llava-phi-3/tokenizer.json",
            "/Users/xddotcom/workspace/rust/llm-playground/models/llava-phi-3/preprocessor_config.json",
        )?;

        let res = llavaphi3mini.get_image_caption(
            "/Users/xddotcom/workspace/rust/llm-playground/models/20240923-173209.jpeg",
        )?;

        println!("{}", res);

        Ok(())
    }
}
