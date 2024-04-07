use crate::{
    utils::{self, normalize},
    Model,
};
use anyhow::{anyhow, bail};
use async_trait::async_trait;
use ndarray::{s, Array1, Axis};
use ort::{CPUExecutionProvider, CoreMLExecutionProvider, GraphOptimizationLevel, Session};
use std::path::Path;
use tokenizers::Tokenizer;

pub struct TextEmbedding {
    model: Session,
    tokenizer: Tokenizer,
    dim: usize,
}

#[async_trait]
impl Model for TextEmbedding {
    type Item = String;
    type Output = Vec<f32>;

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
            let res = self.get_text_embedding(&item).await;
            results.push(res);
        }

        Ok(results)
    }
}

impl TextEmbedding {
    pub async fn new(resources_dir: impl AsRef<Path>) -> anyhow::Result<Self> {
        let download = file_downloader::FileDownload::new(file_downloader::FileDownloadConfig {
            resources_dir: resources_dir.as_ref().to_path_buf(),
            ..Default::default()
        });

        let model_path = download
            .download_if_not_exists("mxbai-embed-large/mxbai-embed-large-v1.onnx")
            .await?;
        let tokenizer_config_path = download
            .download_if_not_exists("mxbai-embed-large/tokenizer.json")
            .await?;

        let model = Session::builder()?
            .with_execution_providers([
                CPUExecutionProvider::default().build(),
                CoreMLExecutionProvider::default().build(),
            ])?
            .with_optimization_level(GraphOptimizationLevel::Level3)?
            .with_intra_threads(16)?
            .with_model_from_file(model_path)?;

        let tokenizer = match Tokenizer::from_file(tokenizer_config_path) {
            Ok(mut tokenizer) => {
                let truncation = tokenizers::utils::truncation::TruncationParams {
                    // TODO check max_length
                    max_length: 512,
                    ..Default::default()
                };
                tokenizer.with_truncation(Some(truncation)).ok();

                Some(tokenizer)
            }
            _ => None,
        }
        .ok_or(anyhow::anyhow!("can not load tokenizer"))?;

        Ok(Self {
            model,
            tokenizer,
            dim: 1024,
        })
    }

    pub async fn get_text_embedding(&self, text: &str) -> anyhow::Result<Vec<f32>> {
        let encoding = self
            .tokenizer
            .encode(text, true)
            .map_err(|err| anyhow!(err))?;
        // let ids: Vec<i64> = encoding.get_ids().iter().map(|&v| v as i64).collect();
        let ids = encoding.get_ids();
        let attention_mask = encoding.get_attention_mask();
        let token_type_ids = encoding.get_type_ids();

        let ids = ndarray::arr1(ids).mapv(|x| x as i64);
        let attention_mask = ndarray::arr1(attention_mask).mapv(|x| x as i64);
        let token_type_ids = ndarray::arr1(token_type_ids).mapv(|x| x as i64);

        let ids = utils::pad_with_zeros(&ids, vec![[0, 512 - ids.len()]]);
        let attention_mask =
            utils::pad_with_zeros(&attention_mask, vec![[0, 512 - attention_mask.len()]]);
        let token_type_ids =
            utils::pad_with_zeros(&token_type_ids, vec![[0, 512 - token_type_ids.len()]]);

        let ids = ids.insert_axis(Axis(0));
        let attention_mask = attention_mask.insert_axis(Axis(0)).clone();
        let token_type_ids = token_type_ids.insert_axis(Axis(0)).clone();

        let outputs = self.model.run(
            ort::inputs!["input_ids" => ids.view(), "attention_mask" => attention_mask.view(), "token_type_ids" => token_type_ids.view()]?,
        )?;

        let output = outputs
            .get("last_hidden_state")
            .ok_or(anyhow!("output not found"))?
            .extract_tensor::<f32>()?
            .view()
            .to_owned();

        // for model mxbai-embed-large-v1, the pooling method is `cls`
        let output = output.slice(s![.., 0, ..]).to_owned();

        let output: Array1<f32> = output.into_shape(self.dim)?.into_dimensionality()?;

        Ok(normalize(output).into_iter().collect())
    }
}

#[test_log::test(tokio::test)]
async fn test_text_embedding() {
    let model = TextEmbedding::new(
        "/Users/zhuo/dev/tezign/bmrlab/tauri-dam-test-playground/apps/desktop/src-tauri/resources",
    )
    .await
    .unwrap();

    let start = std::time::Instant::now();
    let _ = model.get_text_embedding("who are you?").await.unwrap();
    let duration = start.elapsed();
    tracing::info!("Time elapsed in execution is: {:?}", duration);

    let start = std::time::Instant::now();
    let _ = model.get_text_embedding("hello world!").await.unwrap();
    let duration = start.elapsed();
    tracing::info!("Time elapsed in execution is: {:?}", duration);

    let start = std::time::Instant::now();
    let _ = model.get_text_embedding("你是谁").await.unwrap();
    let duration = start.elapsed();
    tracing::info!("Time elapsed in execution is: {:?}", duration);
}
