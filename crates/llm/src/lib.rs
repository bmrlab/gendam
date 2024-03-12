pub(crate) mod cloud;
pub(crate) mod local;
pub mod model;
pub(crate) mod native;

use anyhow::bail;
use async_trait::async_trait;
pub use llama_cpp_2::context::params::LlamaContextParams;
use local::LocalModel;
use native::NativeModel;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::sync::oneshot;

#[derive(Debug)]
pub enum LLMMessage {
    System(String),
    User(String),
    Assistant(String),
}

pub struct LLMParams {
    temperature: Option<f32>,
    seed: Option<u32>,
}

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

#[async_trait]
pub(crate) trait Chat {
    async fn get_completion(
        &self,
        history: Vec<LLMMessage>,
        images: Option<Vec<LLMImageContent>>,
        params: Option<LLMParams>,
    ) -> anyhow::Result<String>;

    fn is_multimodal(&self) -> bool;
}

impl LLM {
    pub async fn new_llama_cpp_model(
        resources_dir: impl AsRef<Path>,
        model: self::model::LlamaCppModel,
        with_server: Option<bool>,
    ) -> anyhow::Result<Self> {
        if with_server.unwrap_or(false) {
            let model = LocalModel::new(resources_dir.as_ref().to_path_buf(), model).await?;

            Ok(Self {
                model: Box::new(model),
            })
        } else {
            let model = NativeModel::new(resources_dir.as_ref().to_path_buf(), model).await?;
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
        if !self.model.is_multimodal() && images.is_some() {
            bail!("image input is not supported for this model");
        }

        self.model.get_completion(history, images, params).await
    }
}

#[test_log::test(tokio::test)]
async fn test_native_llm() {
    let resources_dir =
        "/Users/zhuo/dev/tezign/bmrlab/tauri-dam-test-playground/apps/desktop/src-tauri/resources";
    let llm = LLM::new_llama_cpp_model(
        resources_dir,
        self::model::LlamaCppModel::QWen0_5B,
        Some(false),
    )
    .await
    .unwrap();

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

    println!("{:?}", response);

    assert!(response.is_ok());
}

#[test_log::test(tokio::test)]
async fn test_local_llm() {
    let resources_dir =
        "/Users/zhuo/dev/tezign/bmrlab/tauri-dam-test-playground/apps/desktop/src-tauri/resources";
    let llm = LLM::new_llama_cpp_model(
        resources_dir,
        self::model::LlamaCppModel::QWen0_5B,
        Some(true),
    )
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
