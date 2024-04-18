extern crate accelerate_src;

use anyhow::anyhow;
use candle_core::{backend::BackendDevice, DType, Device, MetalDevice, Tensor};
use candle_transformers::{
    generation::LogitsProcessor,
    models::{moondream, quantized_moondream},
};
use std::path::Path;
use tokenizers::Tokenizer;

#[allow(dead_code)]
pub struct Moondream {
    model: quantized_moondream::Model,
    device: Device,
    tokenizer: Tokenizer,
    logits_processor: LogitsProcessor,
    repeat_penalty: f32,
    repeat_last_n: usize,
}

#[allow(dead_code)]
impl Moondream {
    pub async fn new(
        model_path: impl AsRef<Path>,
        tokenizer_path: impl AsRef<Path>,
    ) -> anyhow::Result<Self> {
        // let (tokenizer_uri, model_uri) = ("moondream/tokenizer.json", "moondream/model-q4_0.gguf");
        // let download = file_downloader::FileDownload::new(file_downloader::FileDownloadConfig {
        //     resources_dir: resources_dir.as_ref().to_path_buf(),
        //     ..Default::default()
        // });
        // let model_path = download.download_if_not_exists(model_uri).await?;
        // let tokenizer_path = download.download_if_not_exists(tokenizer_uri).await?;

        let tokenizer = Tokenizer::from_file(tokenizer_path)
            .map_err(|_| anyhow::anyhow!("failed to initialize tokenizer"))?;

        let logits_processor =
            candle_transformers::generation::LogitsProcessor::new(1337, None, None);

        let device = Device::Metal(MetalDevice::new(0)?);

        let config = moondream::Config::v2();

        let vb = candle_transformers::quantized_var_builder::VarBuilder::from_gguf(
            &model_path,
            &device,
        )?;
        let model = quantized_moondream::Model::new(&config, vb)?;

        Ok(Self {
            model,
            tokenizer,
            device,
            logits_processor,
            repeat_last_n: 64,
            repeat_penalty: 1.0,
        })
    }

    async fn generate(
        &mut self,
        prompt: &str,
        image_path: impl AsRef<Path>,
    ) -> anyhow::Result<String> {
        let image = load_image(image_path)?
            .to_device(&self.device)?
            .to_dtype(DType::F32)?;
        let image_embeds = image.unsqueeze(0)?;
        let image_embeds = image_embeds.apply(self.model.vision_encoder())?;

        let prompt = format!("\n\nQuestion: {0}\n\nAnswer:", prompt);

        // start generation
        let tokens = self
            .tokenizer
            .encode(prompt, true)
            .map_err(|e| anyhow!(e))?;
        if tokens.is_empty() {
            anyhow::bail!("Empty prompts are not supported in the Moondream model.")
        }

        let mut tokens = tokens.get_ids().to_vec();

        // Moondream tokenizer bos_token and eos_token is "<|endoftext|>"
        // https://huggingface.co/vikhyatk/moondream2/blob/main/special_tokens_map.json
        let special_token = match self.tokenizer.get_vocab(true).get("<|endoftext|>") {
            Some(token) => *token,
            None => anyhow::bail!("cannot find the special token"),
        };
        let (bos_token, eos_token) = (special_token, special_token);

        let mut token_ids = vec![];

        // FIXME here 512 is hard-coded
        for index in 0..512 {
            let context_size = if index > 0 { 1 } else { tokens.len() };
            let ctxt = &tokens[tokens.len().saturating_sub(context_size)..];
            let input = Tensor::new(ctxt, &self.device)?.unsqueeze(0)?;
            let logits = if index > 0 {
                self.model.text_model.forward(&input)?
            } else {
                let bos_token = Tensor::new(&[bos_token], &self.device)?.unsqueeze(0)?;
                let logits =
                    self.model
                        .text_model
                        .forward_with_img(&bos_token, &input, &image_embeds)?;

                logits
            };

            let logits = logits.squeeze(0)?.to_dtype(DType::F32)?;
            let logits = if self.repeat_penalty == 1. {
                logits
            } else {
                let start_at = tokens.len().saturating_sub(self.repeat_last_n);
                candle_transformers::utils::apply_repeat_penalty(
                    &logits,
                    self.repeat_penalty,
                    &tokens[start_at..],
                )?
            };
            let next_token = self.logits_processor.sample(&logits)?;
            tokens.push(next_token);
            if next_token == eos_token || tokens.ends_with(&[27, 10619, 29] /* <END> */) {
                break;
            }
            token_ids.push(next_token);
        }

        let result = self.tokenizer.decode(&token_ids, true);

        result
            .map_err(|_| anyhow::anyhow!("failed to generate result"))
            .map(|v| v.trim().to_string())
    }
}

/// Loads an image from disk using the image crate, this returns a tensor with shape
/// (3, 378, 378).
pub fn load_image<P: AsRef<std::path::Path>>(p: P) -> candle_core::Result<Tensor> {
    let img = image::io::Reader::open(p)?
        .decode()
        .map_err(candle_core::Error::wrap)?
        .resize_to_fill(378, 378, image::imageops::FilterType::Triangle); // Adjusted to 378x378
    let img = img.to_rgb8();
    let data = img.into_raw();
    let data = Tensor::from_vec(data, (378, 378, 3), &Device::Cpu)?.permute((2, 0, 1))?;
    let mean = Tensor::new(&[0.5f32, 0.5, 0.5], &Device::Cpu)?.reshape((3, 1, 1))?;
    let std = Tensor::new(&[0.5f32, 0.5, 0.5], &Device::Cpu)?.reshape((3, 1, 1))?;
    (data.to_dtype(candle_core::DType::F32)? / 255.)?
        .broadcast_sub(&mean)?
        .broadcast_div(&std)
}

#[test_log::test(tokio::test)]
async fn test_moondream() {
    let mut moondream = Moondream::new(
        "/Users/zhuo/dev/tezign/bmrlab/tauri-dam-test-playground/apps/desktop/src-tauri/resources",
        "/Users/zhuo/dev/tezign/bmrlab/tauri-dam-test-playground/apps/desktop/src-tauri/resources",
    )
    .await
    .expect("failed to load model");

    let start = std::time::Instant::now();

    match moondream
        .generate("Describe the image.", "/Users/zhuo/Library/Application Support/cc.musedam.local/libraries/cd559e25-c136-4877-825e-84268a53e366/artifacts/69f/69f8e47203029eb5/frames/1000.jpg")
        .await
    {
        Ok(response) => {
            tracing::info!("response: {:?}", response);
        }
        Err(e) => {
            tracing::error!("failed to response: {:?}", e);
        }
    }

    let duration = start.elapsed();
    tracing::info!("Time elapsed in execution is: {:?}", duration);
}
