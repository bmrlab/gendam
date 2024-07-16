extern crate accelerate_src;

use crate::traits::{ImageCaptionInput, ImageCaptionOutput};
use crate::Model;
use anyhow::{anyhow, bail};
use async_trait::async_trait;
use candle_core::backend::BackendDevice;
use candle_core::MetalDevice;
use candle_core::{Device, Tensor};
use candle_transformers::generation::LogitsProcessor;
use candle_transformers::models::blip::VisionConfig;
use candle_transformers::models::quantized_blip;
use candle_transformers::models::{blip, blip_text};
use std::path::Path;
use storage_macro::Storage;
use tokenizers::Tokenizer;
use tracing::debug;

#[derive(Storage)]
pub struct BLIP {
    tokenizer: Tokenizer,
    model: quantized_blip::BlipForConditionalGeneration,
    logits_processor: LogitsProcessor,
    device: Device,
}

const SEP_TOKEN_ID: u32 = 102;

fn blip_base_config() -> blip::Config {
    let text_config = blip_text::Config {
        vocab_size: 30524,
        hidden_size: 768,
        encoder_hidden_size: 768,
        intermediate_size: 3072,
        projection_dim: 768,
        num_hidden_layers: 12,
        num_attention_heads: 12,
        max_position_embeddings: 512,
        hidden_act: candle_nn::Activation::Gelu,
        layer_norm_eps: 1e-12,
        is_decoder: true,
    };
    let vision_config = VisionConfig {
        hidden_size: 768,
        intermediate_size: 3072,
        projection_dim: 512,
        num_hidden_layers: 12,
        num_attention_heads: 12,
        image_size: 384,
        patch_size: 16,
        hidden_act: candle_nn::Activation::Gelu,
        layer_norm_eps: 1e-5,
    };

    blip::Config {
        text_config,
        vision_config,
        projection_dim: 512,
        image_text_hidden_size: 256,
    }
}

pub enum BLIPModel {
    Base,
    Large,
}

impl Model for BLIP {
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
            bail!("too many items");
        }

        let mut results = vec![];

        for item in items {
            let res = self.get_caption(item).await;
            results.push(res);
        }

        Ok(results)
    }
}

impl BLIP {
    pub async fn new(
        model_path: impl AsRef<Path>,
        tokenizer_path: impl AsRef<Path>,
        model_type: BLIPModel,
    ) -> anyhow::Result<Self> {
        let tokenizer = Tokenizer::from_file(tokenizer_path)
            .map_err(|_| anyhow!("failed to initialize tokenizer"))?;

        let logits_processor =
            candle_transformers::generation::LogitsProcessor::new(1337, None, None);

        let config = match model_type {
            BLIPModel::Base => blip_base_config(),
            BLIPModel::Large => blip::Config::image_captioning_large(),
        };

        let device = Device::Metal(MetalDevice::new(0)?);

        let vb = quantized_blip::VarBuilder::from_gguf(model_path, &device)?;
        let model = quantized_blip::BlipForConditionalGeneration::new(&config, vb)?;

        Ok(Self {
            tokenizer,
            model,
            logits_processor,
            device,
        })
    }

    async fn get_caption(&mut self, image_path: impl AsRef<Path>) -> anyhow::Result<String> {
        debug!(
            "generating caption for image: {}",
            image_path.as_ref().display()
        );
        let image = self
            .load_image(image_path.as_ref())?
            .to_device(&self.device)?;
        let image_embeds = image.unsqueeze(0)?.apply(self.model.vision_model())?;

        let mut token_ids = vec![30522u32];

        // we need this to make multi time generation work
        self.model.text_decoder().reset_kv_cache();

        for index in 0..1000 {
            let context_size = if index > 0 { 1 } else { token_ids.len() };
            let start_pos = token_ids.len().saturating_sub(context_size);
            let input_ids = Tensor::new(&token_ids[start_pos..], &self.device)?.unsqueeze(0)?;
            let logits = self
                .model
                .text_decoder()
                .forward(&input_ids, &image_embeds)?;
            let logits = logits.squeeze(0)?;
            let logits = logits.get(logits.dim(0)? - 1)?;
            let token = self.logits_processor.sample(&logits)?;
            if token == SEP_TOKEN_ID {
                break;
            }
            token_ids.push(token);
        }

        let result = self.tokenizer.decode(&token_ids, true);

        result.map_err(|_| anyhow!("failed to generate caption"))
    }

    pub fn load_image<P: AsRef<std::path::Path>>(&self, p: P) -> candle_core::Result<Tensor> {
        let data = self
            .read_blocking(p.as_ref().to_path_buf())
            .map_err(candle_core::Error::wrap)?;
        let img = image::io::Reader::new(std::io::Cursor::new(data.to_vec()))
            .with_guessed_format()?
            .decode()
            .map_err(candle_core::Error::wrap)?
            .resize_to_fill(384, 384, image::imageops::FilterType::Triangle);
        let img = img.to_rgb8();
        let data = img.into_raw();
        let data = Tensor::from_vec(data, (384, 384, 3), &Device::Cpu)?.permute((2, 0, 1))?;
        let mean = Tensor::new(&[0.48145466f32, 0.4578275, 0.40821073], &Device::Cpu)?
            .reshape((3, 1, 1))?;
        let std = Tensor::new(&[0.26862954f32, 0.261_302_6, 0.275_777_1], &Device::Cpu)?
            .reshape((3, 1, 1))?;
        (data.to_dtype(candle_core::DType::F32)? / 255.)?
            .broadcast_sub(&mean)?
            .broadcast_div(&std)
    }
}

#[test_log::test(tokio::test)]
async fn test_caption() {
    let blip = BLIP::new(
        "/Users/zhuo/dev/tezign/bmrlab/tauri-dam-test-playground/apps/desktop/src-tauri/resources",
        "/Users/zhuo/dev/tezign/bmrlab/tauri-dam-test-playground/apps/desktop/src-tauri/resources",
        BLIPModel::Base,
    )
    .await;

    match blip {
        Ok(mut blip) => {
            tracing::info!("start execution");
            let start = std::time::Instant::now();

            let frame_paths: Vec<String> = std::fs::read_dir("/Users/zhuo/Library/Application Support/cc.musedam.local/libraries/5d2fa67e-4c9a-40fa-9304-3f61f7836044/artifacts/7e05d9f79842116c/frames")
                .unwrap()
                .map(|res|   res.map(|e| e.path()))
                .collect::<Result<Vec<_>, std::io::Error>>()
                .unwrap()
                .iter()
                .filter_map(|v| {
                    if v.extension() == Some("jpg".as_ref()) { Some(v.to_str().unwrap().to_string()) }
                    else {None}
                })
                .collect();

            for path in frame_paths {
                let temp_start = std::time::Instant::now();

                let caption = blip.get_caption(path).await;
                debug!("caption: {:?}", caption);
                assert!(caption.is_ok());

                let duration = temp_start.elapsed();
                tracing::info!("Time elapsed in execution is: {:?}", duration);
            }

            let duration = start.elapsed();
            tracing::info!("Time elapsed in execution is: {:?}", duration);
        }
        Err(e) => {
            tracing::error!("failed to load blip: {}", e);
        }
    }
}
