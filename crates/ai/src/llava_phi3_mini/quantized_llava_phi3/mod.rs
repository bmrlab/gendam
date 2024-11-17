mod clip;
mod config;
mod image_processor;
mod linear;
mod llava;
mod sequential;

use self::{
    image_processor::{HFPreProcessorConfig, ImageProcessor},
    llava::{format_prompt, QLLaVAPhi3},
};
use crate::llm::candle::TokenOutputStream;
use candle_core::{Device, IndexOp, Tensor};
use candle_transformers::generation::{LogitsProcessor, Sampling};
use std::path::Path;

/// The length of the sample to generate (in tokens).
const SAMPLE_LEN: usize = 1000;
const REPEAT_PENALTY: f32 = 1.1;
const REPEAT_LAST_N: usize = 64;

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
        #[cfg(debug_assertions)]
        {
            println!("");
        }
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
            if let Some(_token) = tos.next_token(next_token)? {
                #[cfg(debug_assertions)]
                {
                    use std::io::Write;
                    print!("{}", _token);
                    std::io::stdout().flush()?;
                }
            }
        }
        #[cfg(debug_assertions)]
        {
            println!("\n");
        }

        let result = tos.decode_all()?;
        Ok(result)
    }

    fn assemble_grid_images(
        &mut self,
        image_file_paths: &Vec<impl AsRef<Path>>,
    ) -> anyhow::Result<(usize, usize, image::DynamicImage)> {
        // Compute the optimal grid layout for the images
        let num_images = image_file_paths.len();
        let grid_cols = (num_images as f64).sqrt().ceil() as usize;
        let grid_rows = (num_images + grid_cols - 1) / grid_cols;

        // Make the grid square by using the larger dimension
        let grid_size = grid_cols.max(grid_rows);

        // Set target dimensions for each grid cell (square)
        let target_size = 2000u32;
        let border_size = 2u32;

        let mut grid_images = vec![];
        for path in image_file_paths {
            let img = image::ImageReader::open(path)?
                .with_guessed_format()?
                .decode()?;

            // Calculate resize dimensions while maintaining aspect ratio
            let (width, height) = (img.width(), img.height());
            let ratio = width as f32 / height as f32;
            let (new_width, new_height) = if ratio > 1.0 {
                (target_size, (target_size as f32 / ratio) as u32)
            } else {
                ((target_size as f32 * ratio) as u32, target_size)
            };

            let resized = img.resize(new_width, new_height, image::imageops::FilterType::Lanczos3);
            grid_images.push(resized);
        }

        // Calculate total grid dimensions including borders for square grid
        let grid_width = target_size * grid_size as u32 + border_size * (grid_size as u32 + 1);
        let grid_height = target_size * grid_size as u32 + border_size * (grid_size as u32 + 1);

        // Create a white background grid
        let mut grid = image::RgbaImage::from_pixel(
            grid_width,
            grid_height,
            image::Rgba([255, 255, 255, 255]),
        );

        for (i, img) in grid_images.iter().enumerate() {
            let grid_x = (i % grid_cols) as u32;
            let grid_y = (i / grid_cols) as u32;

            // Calculate center position for the image within its grid cell
            let cell_x = grid_x * (target_size + border_size) + border_size;
            let cell_y = grid_y * (target_size + border_size) + border_size;

            let x = cell_x + (target_size - img.width()) / 2;
            let y = cell_y + (target_size - img.height()) / 2;

            image::imageops::overlay(&mut grid, img, x as i64, y as i64);
        }

        Ok((grid_cols, grid_rows, image::DynamicImage::ImageRgba8(grid)))
    }

    /// llava phi 3 虽然支持多个图片输入，但是并不能正常输出结果
    /// 模型推理方法保留支持多图的逻辑，但实际是把图片拼接成一个大图，作为1张图片输入
    #[tracing::instrument(level = "info", skip(self, image_file_paths))]
    pub fn get_image_caption(
        &mut self,
        image_file_paths: Vec<impl AsRef<Path>>,
        prompt: Option<String>,
    ) -> anyhow::Result<String> {
        let (_grid_cols, _grid_rows, img) = self.assemble_grid_images(&image_file_paths)?;
        let image_tensor = self
            .image_processor
            .preprocess(&img)?
            .unsqueeze(0)?
            // .to_dtype(candle_core::DType::BF16)?
            .to_device(&self.device)?;
        let image_size = (img.width(), img.height());
        // img.save("assembled_grid_images.png")?;

        let mut prompt = prompt.unwrap_or("Please describe the image".to_string());
        if image_file_paths.len() > 1 {
            // let num_images = image_file_paths.len();
            prompt = format!("The input consists of a sequence of consecutive images that we'll analyze together.\n{prompt}");
        }
        let prompt_str = format_prompt(prompt.as_str(), 1);
        let prompt_tokens = self
            .inner
            .tokenizer_image_token(&self.device, prompt_str.as_str())?;

        let input_embeds = self.inner.prepare_inputs_labels_for_multimodal(
            &self.device,
            &prompt_tokens,
            &[image_tensor],
            &[image_size],
        )?;

        // TODO: seed and temperature should be configurable
        let caption = self.generate(input_embeds, 299792458, 0.)?;

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

        let image_file_paths = vec![
            "/Users/xddotcom/local_dam_files/明星影视剧画面/6000.jpg",
            // "/Users/xddotcom/local_dam_files/明星影视剧画面/1170000.jpg",
            // "/Users/xddotcom/local_dam_files/明星影视剧画面/392000.jpg",
            // "/Users/xddotcom/local_dam_files/明星影视剧画面/304000.jpg",
        ];
        let prompt = r#"You are an advanced image description expert. Examine this image and describe the visual content. Pay attention to: people's actions and expressions, scene changes, movement, and any key events or transitions. Begin your response with 'The image ...'. Limit your response to no more than 50 words."#.to_string();
        // let prompt = format!(
        //     r#"You are an advanced video description expert. Watch this image sequence of video clip consisting of {num_images} frames, narrate what you see and describe any notable changes between frames. Begin your response with 'The video clip...'. Limit your response to no more than 50 words."#,
        //     num_images = image_file_paths.len()
        // );
        let res = llavaphi3mini.get_image_caption(image_file_paths, Some(prompt))?;

        println!("{}", res);

        Ok(())
    }
}
