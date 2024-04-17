use crate::{
    ort::load_onnx_model,
    traits::{TextEmbeddingInput, TextEmbeddingOutput},
    utils::{self, normalize},
    Model,
};
use anyhow::anyhow;
use async_trait::async_trait;
use ndarray::{Array1, Axis};
use ort::Session;
use std::path::Path;
use tokenizers::Tokenizer;

pub struct OrtTextEmbedding {
    model: Session,
    tokenizer: Tokenizer,
    dim: usize,
    max_len: usize,
}

impl OrtTextEmbedding {
    pub async fn new(resources_dir: impl AsRef<Path>) -> anyhow::Result<Self> {
        let download = file_downloader::FileDownload::new(file_downloader::FileDownloadConfig {
            resources_dir: resources_dir.as_ref().to_path_buf(),
            ..Default::default()
        });

        let model_path = download
            .download_if_not_exists("puff-base-v1/model_quantized.onnx")
            .await?;
        let tokenizer_config_path = download
            .download_if_not_exists("puff-base-v1/tokenizer.json")
            .await?;

        let model = load_onnx_model(model_path, None)?;

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
            max_len: 512,
        })
    }

    pub async fn get_text_embedding(&self, text: &str) -> anyhow::Result<Vec<f32>> {
        let encoding = self
            .tokenizer
            .encode(text, true)
            .map_err(|err| anyhow!(err))?;

        let ids = encoding.get_ids();
        let attention_mask = encoding.get_attention_mask();
        // let token_type_ids = encoding.get_type_ids();

        let ids = ndarray::arr1(ids).mapv(|x| x as i64);
        let attention_mask = ndarray::arr1(attention_mask).mapv(|x| x as i64);
        // let token_type_ids = ndarray::arr1(token_type_ids).mapv(|x| x as i64);

        let ids = utils::pad_with_zeros(&ids, vec![[0, self.max_len - ids.len()]]);
        let attention_mask = utils::pad_with_zeros(
            &attention_mask,
            vec![[0, self.max_len - attention_mask.len()]],
        );
        // let token_type_ids = utils::pad_with_zeros(
        //     &token_type_ids,
        //     vec![[0, self.max_len - token_type_ids.len()]],
        // );

        let ids = ids.insert_axis(Axis(0));
        let attention_mask = attention_mask.insert_axis(Axis(0)).clone();
        // let token_type_ids = token_type_ids.insert_axis(Axis(0)).clone();

        let outputs = self.model.run(
            ort::inputs!["input_ids" => ids.view(), "attention_mask" => attention_mask.view()]?,
        )?;

        let output = outputs
            .get("sentence_embedding")
            .ok_or(anyhow!("output not found"))?
            .try_extract_tensor::<f32>()?
            .view()
            .to_owned();

        // for model mxbai-embed-large-v1, the pooling method is `cls`
        // let output = output.slice(s![.., 0, ..]).to_owned();

        let output: Array1<f32> = output.into_shape(self.dim)?.into_dimensionality()?;

        Ok(normalize(output).into_iter().collect())
    }
}

#[async_trait]
impl Model for OrtTextEmbedding {
    type Item = TextEmbeddingInput;
    type Output = TextEmbeddingOutput;

    fn batch_size_limit(&self) -> usize {
        1
    }

    async fn process(
        &mut self,
        items: Vec<String>,
    ) -> anyhow::Result<Vec<anyhow::Result<Vec<f32>>>> {
        let mut results = vec![];

        for item in items {
            let res = self.get_text_embedding(&item).await;
            results.push(res);
        }

        Ok(results)
    }
}

#[test_log::test(tokio::test)]
async fn test_text_embedding() {
    let model = OrtTextEmbedding::new(
        "/Users/zhuo/dev/tezign/bmrlab/tauri-dam-test-playground/apps/desktop/src-tauri/resources",
    )
    .await
    .unwrap();

    let start = std::time::Instant::now();
    let embed1 = model.get_text_embedding("who are you?").await.unwrap();
    let duration = start.elapsed();
    tracing::info!("Time elapsed in execution is: {:?}", duration);

    let start = std::time::Instant::now();
    let embed2 = model.get_text_embedding("hello world!").await.unwrap();
    let duration = start.elapsed();
    tracing::info!("Time elapsed in execution is: {:?}", duration);

    let start = std::time::Instant::now();
    let embed3 = model.get_text_embedding("你是谁").await.unwrap();
    let duration = start.elapsed();
    tracing::info!("Time elapsed in execution is: {:?}", duration);

    // compare cosine similarity
    let sim1: f32 = embed1.iter().zip(embed2.iter()).map(|(x, y)| x * y).sum();
    tracing::info!("sim 1 and 2: {:?}", sim1);
    let sim2: f32 = embed1.iter().zip(embed3.iter()).map(|(x, y)| x * y).sum();
    tracing::info!("sim 1 and 3: {:?}", sim2);

    assert!(sim1 < sim2);
    assert!(embed1.len() == 1024);
}
