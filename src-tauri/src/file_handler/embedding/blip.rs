extern crate accelerate_src;

use std::path::Path;

use super::token_output_stream::TokenOutputStream;
use anyhow::anyhow;
use candle_core::backend::BackendDevice;
use candle_core::MetalDevice;
use candle_core::{Device, Tensor};
use candle_transformers::generation::LogitsProcessor;
use candle_transformers::models::blip;
use candle_transformers::models::quantized_blip;
use tokenizers::Tokenizer;
use tracing::{debug, error};

pub struct BLIP {
    tokenizer: TokenOutputStream,
    model: quantized_blip::BlipForConditionalGeneration,
    logits_processor: LogitsProcessor,
    device: Device,
}

const SEP_TOKEN_ID: u32 = 102;

impl BLIP {
    pub fn new(model_dir: impl AsRef<Path>) -> anyhow::Result<Self> {
        let tokenizer_path = model_dir.as_ref().join("tokenizer.json");
        debug!("loading tokenizer: {}", tokenizer_path.display());

        let model_path = model_dir
            .as_ref()
            .join("blip-image-captioning-large-q4k.gguf");
        debug!("loading model: {}", model_path.display());

        let tokenizer = Tokenizer::from_file(tokenizer_path)
            .map_err(|_| anyhow!("failed to initialize tokenizer"))?;
        let tokenizer = TokenOutputStream::new(tokenizer);

        let logits_processor =
            candle_transformers::generation::LogitsProcessor::new(1337, None, None);

        let config = blip::Config::image_captioning_large();

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

    pub async fn get_caption(&mut self, image_path: impl AsRef<Path>) -> anyhow::Result<String> {
        debug!(
            "generating caption for image: {}",
            image_path.as_ref().display()
        );
        let image = load_image(image_path.as_ref())?.to_device(&self.device)?;
        let image_embeds = image.unsqueeze(0)?.apply(self.model.vision_model())?;

        let mut token_ids = vec![30522u32];
        let mut result = vec![];

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
            if let Some(t) = self.tokenizer.next_token(token)? {
                result.push(t);
            }
        }

        Ok(result.join(""))
    }
}

pub fn load_image<P: AsRef<std::path::Path>>(p: P) -> candle_core::Result<Tensor> {
    let img = image::io::Reader::open(p)?
        .decode()
        .map_err(candle_core::Error::wrap)?
        .resize_to_fill(384, 384, image::imageops::FilterType::Triangle);
    let img = img.to_rgb8();
    let data = img.into_raw();
    let data = Tensor::from_vec(data, (384, 384, 3), &Device::Cpu)?.permute((2, 0, 1))?;
    let mean =
        Tensor::new(&[0.48145466f32, 0.4578275, 0.40821073], &Device::Cpu)?.reshape((3, 1, 1))?;
    let std = Tensor::new(&[0.26862954f32, 0.261_302_6, 0.275_777_1], &Device::Cpu)?
        .reshape((3, 1, 1))?;
    (data.to_dtype(candle_core::DType::F32)? / 255.)?
        .broadcast_sub(&mean)?
        .broadcast_div(&std)
}

#[test_log::test(tokio::test)]
async fn test_caption() {
    let blip = BLIP::new("/Users/zhuo/dev/bmrlab/tauri-dam-test-playground/src-tauri/resources/blip");

    assert!(blip.is_ok());
    let mut blip = blip.unwrap();

    let caption = blip.get_caption("/Users/zhuo/Library/Application Support/cc.musedam.local/1aaa451c0bee906e2d1f9cac21ebb2ef5f2f82b2f87ec928fc04b58cbceda60b/frames/15000000.png").await;
    debug!("caption: {:?}", caption);
    assert!(caption.is_ok());
}
