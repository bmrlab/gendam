use crate::Model;

use super::{preprocess, utils};
use anyhow::{anyhow, bail};
use async_trait::async_trait;
use image::RgbImage;
pub use model::*;
use ndarray::{Array1, Axis};
use ort::{CPUExecutionProvider, CoreMLExecutionProvider, GraphOptimizationLevel, Session};
use std::path::{Path, PathBuf};
use tokenizers::tokenizer::Tokenizer;
use utils::normalize;

pub mod model;

pub struct CLIP {
    image_model: Option<Session>,
    text_model: Option<Session>,
    text_tokenizer: Option<Tokenizer>,
    dim: usize,
}

type CLIPEmbedding = Array1<f32>;

#[derive(Clone)]
pub enum CLIPInput {
    Image(RgbImage),
    ImageFilePath(PathBuf),
    Text(String),
}

#[async_trait]
impl Model for CLIP {
    type Item = CLIPInput;
    type Output = CLIPEmbedding;

    fn batch_size_limit(&self) -> usize {
        // TODO 后续可以支持 batch 模式
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
            let res = match item {
                CLIPInput::Image(rgb) => self.get_image_embedding_from_image(&rgb).await,
                CLIPInput::ImageFilePath(path) => self.get_image_embedding_from_file(&path).await,
                CLIPInput::Text(text) => self.get_text_embedding(&text).await,
            };

            results.push(res);
        }

        Ok(results)
    }
}

impl CLIP {
    pub async fn new(
        model: model::CLIPModel,
        resources_dir: impl AsRef<Path>,
    ) -> anyhow::Result<Self> {
        let (image_model_uri, text_model_uri, text_tokenizer_vocab_uri) = model.model_uri();
        let dim = model.dim();

        let download = file_downloader::FileDownload::new(file_downloader::FileDownloadConfig {
            resources_dir: resources_dir.as_ref().to_path_buf(),
            ..Default::default()
        });

        let image_model_path = download.download_if_not_exists(&image_model_uri).await?;
        let text_model_path = download.download_if_not_exists(&text_model_uri).await?;
        let text_tokenizer_vocab_path = download
            .download_if_not_exists(&text_tokenizer_vocab_uri)
            .await?;

        Self::from_file(
            image_model_path,
            text_model_path,
            text_tokenizer_vocab_path,
            dim,
        )
    }

    pub fn from_file(
        image_model_path: impl AsRef<Path>,
        text_model_path: impl AsRef<Path>,
        text_tokenizer_vocab_path: impl AsRef<Path>,
        dim: usize,
    ) -> anyhow::Result<Self> {
        let image_model = Session::builder()?
            .with_execution_providers([
                CPUExecutionProvider::default().build(),
                CoreMLExecutionProvider::default().build(),
            ])?
            .with_optimization_level(GraphOptimizationLevel::Level3)?
            .with_intra_threads(16)?
            .with_model_from_file(image_model_path)?;

        let text_model = Session::builder()?
            .with_execution_providers([
                CPUExecutionProvider::default().build(),
                CoreMLExecutionProvider::default().build(),
            ])?
            .with_optimization_level(GraphOptimizationLevel::Level3)?
            .with_intra_threads(16)?
            .with_model_from_file(text_model_path)?;

        let text_tokenizer = match Tokenizer::from_file(text_tokenizer_vocab_path) {
            Ok(mut tokenizer) => {
                let truncation = tokenizers::utils::truncation::TruncationParams {
                    // default CLIP text truncation
                    max_length: 77,
                    ..Default::default()
                };
                tokenizer.with_truncation(Some(truncation)).ok();

                Some(tokenizer)
            }
            _ => None,
        };

        Ok(Self {
            image_model: Some(image_model),
            text_model: Some(text_model),
            text_tokenizer,
            dim,
        })
    }

    /// Preprocess image and get embedding (in size 1 * DIM)
    ///
    /// # Arguments
    ///
    /// * `image_path` - input image path
    pub async fn get_image_embedding_from_file(
        &self,
        image_path: impl AsRef<Path>,
    ) -> anyhow::Result<CLIPEmbedding> {
        let image = preprocess::read_image(image_path)?;
        self.get_image_embedding_from_image(&image).await
    }

    pub async fn get_image_embedding_from_image(
        &self,
        image: &RgbImage,
    ) -> anyhow::Result<CLIPEmbedding> {
        let image_model = self
            .image_model
            .as_ref()
            .ok_or(anyhow!("image model not found"))?;

        let image = preprocess::preprocess_rgb8_image(image)?;

        // add axis to reshape to (1, C, H, W)
        let image = image.insert_axis(Axis(0)).clone();
        let outputs = image_model.run(ort::inputs!["pixel_values" => image.view()]?)?;

        let output = outputs
            .get("output")
            .ok_or(anyhow!("output not found"))?
            .extract_tensor::<f32>()?
            .view()
            .to_owned();

        let output: CLIPEmbedding = output.into_shape(self.dim)?.into_dimensionality()?;

        Ok(normalize(output))
    }

    pub async fn get_text_embedding(&self, text: &str) -> anyhow::Result<CLIPEmbedding> {
        let model = self
            .text_model
            .as_ref()
            .ok_or(anyhow!("text model not found"))?;
        let tokenizer = self
            .text_tokenizer
            .as_ref()
            .ok_or(anyhow!("text tokenizer not found"))?;

        let encoding = tokenizer.encode(text, true).map_err(|err| anyhow!(err))?;

        let ids = encoding.get_ids();
        let attention_mask = encoding.get_attention_mask();
        let ids = ndarray::arr1(&ids).mapv(|x| x as i32);
        let attention_mask = ndarray::arr1(&attention_mask).mapv(|x| x as i32);
        // add padding
        let ids = utils::pad_with_zeros(&ids, vec![[0, 77 - ids.len()]]);
        let attention_mask =
            utils::pad_with_zeros(&attention_mask, vec![[0, 77 - attention_mask.len()]]);
        // add axis
        let ids = ids.insert_axis(Axis(0)).clone();
        let attention_mask = attention_mask.insert_axis(Axis(0)).clone();

        let outputs = model.run(
            ort::inputs!["input_ids" => ids.view(), "attention_mask" => attention_mask.view()]?,
        )?;

        let output = outputs
            .get("output")
            .ok_or(anyhow!("output not found"))?
            .extract_tensor::<f32>()?
            .view()
            .to_owned();

        let output: CLIPEmbedding = output.into_shape(self.dim)?.into_dimensionality()?;

        Ok(normalize(output))
    }

    pub fn dim(&self) -> usize {
        self.dim
    }
}

#[test_log::test(tokio::test)]
async fn test_async_clip() {
    let clip = CLIP::from_file(
        "/Users/zhuo/dev/bmrlab/tauri-dam-test-playground/target/debug/resources/CLIP-ViT-B-32-laion2B-s34B-b79K/visual.onnx",
        "/Users/zhuo/dev/bmrlab/tauri-dam-test-playground/target/debug/resources/CLIP-ViT-B-32-laion2B-s34B-b79K/textual.onnx",
        "/Users/zhuo/dev/bmrlab/tauri-dam-test-playground/target/debug/resources/CLIP-ViT-B-32-laion2B-s34B-b79K/tokenizer.json",
        512,
    )
    .unwrap();

    let clip = tokio::sync::RwLock::new(clip);
    let clip = std::sync::Arc::new(clip);

    let paths = vec!["/Users/zhuo/Desktop/avatar.JPG"];

    for path in paths {
        let path = path.to_string();
        let clip = std::sync::Arc::clone(&clip);
        tokio::spawn(async move {
            tracing::debug!("{:?}", path);
            let embedding = clip.read().await.get_image_embedding_from_file(path).await;
            match embedding {
                Ok(vector) => {
                    tracing::debug!("{:?}", vector);
                    tracing::debug!("square sum: {:?}", vector.dot(&vector));
                }
                Err(e) => {
                    tracing::error!("{:?}", e);
                }
            }
        });
    }

    tokio::task::yield_now().await;
}
