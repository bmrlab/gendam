pub(crate) mod cloud;
#[cfg(feature = "local")]
pub(crate) mod local;
pub mod model;
#[cfg(feature = "native")]
pub(crate) mod native;

use anyhow::bail;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::sync::oneshot;

#[cfg(feature = "native")]
pub use llama_cpp_2::context::params::LlamaContextParams;
#[cfg(feature = "local")]
use local::LocalModel;
#[cfg(feature = "native")]
use native::NativeModel;

#[derive(Debug)]
pub enum LLMMessage {
    System(String),
    User(String),
    Assistant(String),
}

pub struct LLMParams {
    #[allow(dead_code)]
    temperature: Option<f32>,
    seed: Option<u32>,
}

#[cfg(feature = "native")]
impl Into<LlamaContextParams> for LLMParams {
    fn into(self) -> LlamaContextParams {
        let params = LlamaContextParams::default();
        if let Some(seed) = self.seed {
            params.with_seed(seed)
        } else {
            params
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LLMImageContent {
    #[serde(rename = "data")]
    base64: String,
    id: usize,
}

#[allow(dead_code)]
pub struct LLMInput {
    history: Vec<LLMMessage>,
    images: Vec<LLMImageContent>,
    params: Option<LLMParams>,
    tx: oneshot::Sender<anyhow::Result<String>>,
}

pub enum LLMPayload {
    Input(LLMInput),
    Stop,
}

pub struct LLM {
    model: Box<dyn Chat>,
}

#[allow(dead_code)]
pub struct LLMEmbedding {
    model: Box<dyn Embedding>
}

#[async_trait]
pub(crate) trait Chat {
    async fn get_completion(
        &self,
        history: Vec<LLMMessage>,
        images: Option<Vec<LLMImageContent>>,
        params: Option<LLMParams>,
    ) -> anyhow::Result<String>;
    fn is_multimodal_chat(&self) -> bool;
}

#[async_trait]
pub(crate) trait Embedding {
    async fn get_embedding(
        &self,
        prompt: String,
        images: Option<Vec<LLMImageContent>>,
        params: Option<LLMParams>,
    ) -> anyhow::Result<Vec<f32>>;
    fn is_multimodal_embedding(&self) -> bool;
}

impl LLM {
    pub async fn new_llama_cpp_model(
        resources_dir: impl AsRef<Path>,
        model: self::model::LlamaCppModel,
    ) -> anyhow::Result<Self> {
        #[cfg(feature = "native")]
        {
            let model = NativeModel::new(resources_dir.as_ref().to_path_buf(), model).await?;
            return Ok(Self {
                model: Box::new(model),
            });
        }

        #[cfg(feature = "local")]
        {
            let model = LocalModel::new(resources_dir.as_ref().to_path_buf(), model).await?;

            Ok(Self {
                model: Box::new(model),
            })
        }
    }

    pub async fn get_completion(
        &self,
        history: Vec<LLMMessage>,
        images: Option<Vec<LLMImageContent>>,
        params: Option<LLMParams>,
    ) -> anyhow::Result<String> {
        if !self.model.is_multimodal_chat() && images.is_some() {
            bail!("image input is not supported for this model");
        }

        self.model.get_completion(history, images, params).await
    }
}

#[test_log::test(tokio::test)]
async fn test_llm() {
    let resources_dir =
        "/Users/zhuo/dev/tezign/bmrlab/tauri-dam-test-playground/apps/desktop/src-tauri/resources";
    let llm = LLM::new_llama_cpp_model(resources_dir, self::model::LlamaCppModel::QWen0_5B)
        .await
        .unwrap();

    let temp_start = std::time::Instant::now();

    let prompt = r#"You will be provided a list of visual details observed at regular intervals, along with an audio description.
These pieces of information originate from a single video.
The visual details are extracted from the video at fixed time intervals and represent consecutive frames.
Typically, the video consists of a brief sequence showing one or more subjects...

Please note that the following list of image descriptions (visual details) was obtained by extracting individual frames from a continuous video featuring one or more subjects.
Depending on the case, all depicted individuals may correspond to the same person(s), with minor variations due to changes in lighting, angle, and facial expressions over time.
Regardless, assume temporal continuity among the frames unless otherwise specified.

Here are the descriptions:

a close up of a cell phone with pictures of people on it
a close up of a cell phone with pictures of people on it
a close up of a cell phone with pictures of people on it
a close up of a cell phone with pictures of people on it
a close up of a cell phone with pictures of people on it"#;

    let response = llm
        .get_completion(vec![LLMMessage::User(prompt.into())], None, None)
        .await;

    tracing::info!("{:?}", response);

    let duration = temp_start.elapsed();
    tracing::info!("Time elapsed in execution is: {:?}", duration);

    assert!(response.is_ok());
}
