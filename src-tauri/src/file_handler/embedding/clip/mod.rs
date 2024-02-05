use super::{preprocess, utils};
use anyhow::anyhow;
use image::RgbImage;
use ndarray::Axis;
use ort::{GraphOptimizationLevel, Session};
use std::path::Path;
use tokenizers::tokenizer::Tokenizer;
use tracing::debug;
pub mod model;

pub struct CLIP {
    image_model: Option<Session>,
    text_model: Option<Session>,
    text_tokenizer: Option<Tokenizer>,
}

type CLIPEmbedding = ndarray::Array2<f32>;

impl CLIP {
    pub async fn new(
        model: model::CLIPModel,
        resources_dir: impl AsRef<Path>,
    ) -> anyhow::Result<Self> {
        let (image_model_uri, text_model_uri, text_tokenizer_vocab_uri) = {
            match model {
                model::CLIPModel::ViTB32 => {
                    let model_uri = std::path::Path::new("CLIP-ViT-B-32-laion2B-s34B-b79K");
                    (
                        model_uri.join("visual.onnx"),
                        model_uri.join("textual.onnx"),
                        model_uri.join("tokenizer.json"),
                    )
                }
                model::CLIPModel::ViTL14 => {
                    todo!("add model info for ViT-L/14")
                }
            }
        };

        let download = crate::download::FileDownload::new(crate::download::FileDownloadConfig {
            resources_dir: resources_dir.as_ref().to_path_buf(),
            ..Default::default()
        });

        let image_model_path = download.download_if_not_exists(&image_model_uri).await?;
        let text_model_path = download.download_if_not_exists(&text_model_uri).await?;
        let text_tokenizer_vocab_path = download
            .download_if_not_exists(&text_tokenizer_vocab_uri)
            .await?;

        Self::from_file(image_model_path, text_model_path, text_tokenizer_vocab_path)
    }

    pub fn from_file(
        image_model_path: impl AsRef<Path>,
        text_model_path: impl AsRef<Path>,
        text_tokenizer_vocab_path: impl AsRef<Path>,
    ) -> anyhow::Result<Self> {
        let image_model = Session::builder()?
            .with_optimization_level(GraphOptimizationLevel::Level3)?
            .with_intra_threads(1)?
            .with_model_from_file(image_model_path)?;

        let text_model = Session::builder()?
            .with_optimization_level(GraphOptimizationLevel::Level3)?
            .with_intra_threads(1)?
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

        Ok(output.into_dimensionality()?)
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

        debug!("{:?}", ids);
        debug!("{:?}", attention_mask);

        let outputs = model.run(
            ort::inputs!["input_ids" => ids.view(), "attention_mask" => attention_mask.view()]?,
        )?;

        let output = outputs
            .get("output")
            .ok_or(anyhow!("output not found"))?
            .extract_tensor::<f32>()?
            .view()
            .to_owned();

        Ok(output.into_dimensionality()?)
    }
}

#[test_log::test(tokio::test)]
async fn test_async_clip() {
    let clip = CLIP::from_file(
        "./resources/visual.onnx",
        "./resources/textual.onnx",
        "./resources/tokenizer.json",
    )
    .unwrap();

    let clip = tokio::sync::RwLock::new(clip);
    let clip = std::sync::Arc::new(clip);

    let paths = vec!["/Users/zhuo/Library/Application Support/cc.musedam.local/1aaa451c0bee906e2d1f9cac21ebb2ef5f2f82b2f87ec928fc04b58cbceda60b/frames/89000000.png", "/Users/zhuo/Library/Application Support/cc.musedam.local/1aaa451c0bee906e2d1f9cac21ebb2ef5f2f82b2f87ec928fc04b58cbceda60b/frames/90000000.png"];

    let mut set = tokio::task::JoinSet::new();

    for path in paths {
        let path = path.to_string();
        let clip = std::sync::Arc::clone(&clip);
        set.spawn(async move {
            debug!("{:?}", path);
            let _ = clip.read().await.get_image_embedding_from_file(path).await;
        });
    }

    while let Some(res) = set.join_next().await {
        debug!("{:?}", res);
    }
}
