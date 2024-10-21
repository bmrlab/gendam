mod quantized_llava_phi3;
use quantized_llava_phi3::{format_prompt, load_image, QLLaVAPhi3};

use crate::traits::{ImageCaptionInput, ImageCaptionOutput, Model};

use candle_core::Device;
use std::path::Path;

impl Model for LLaVAPhi3Mini {
    type Item = ImageCaptionInput;
    type Output = ImageCaptionOutput;

    fn batch_size_limit(&self) -> usize {
        1
    }

    async fn process(
        &mut self,
        _items: Vec<Self::Item>,
    ) -> anyhow::Result<Vec<anyhow::Result<Self::Output>>> {
        let results: Vec<Result<String, anyhow::Error>> = vec![];
        Ok(results)
    }
}

pub struct LLaVAPhi3Mini {
    qllavaphi3: QLLaVAPhi3,
}

impl LLaVAPhi3Mini {
    pub async fn new(
        device: &str,
        gguf_model_path: impl AsRef<Path>,
        mmproj_gguf_model_path: impl AsRef<Path>,
        tokenizer_path: impl AsRef<Path>,
    ) -> anyhow::Result<Self> {
        let device = match device {
            "metal" => Device::new_metal(0)?,
            _ => anyhow::bail!("Unsupported device: {}", device),
        };
        let qllavaphi3 = QLLaVAPhi3::load(
            &device,
            gguf_model_path,
            mmproj_gguf_model_path,
            tokenizer_path,
        )?;
        Ok(Self {
            //
            qllavaphi3,
        })
    }
}
