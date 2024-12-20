use super::traits::{LLMInput, LLMOutput, Model};
pub mod candle;
pub mod native;
pub mod openai;
pub mod qllama;
pub mod qwen2;

#[derive(Debug, Clone)]
pub enum LLMUserMessage {
    Text(String),
    ImageUrl(String),
}

#[derive(Debug, Clone)]
pub enum LLMMessage {
    System(String),
    User(Vec<LLMUserMessage>),
    Assistant(String),
}

impl LLMMessage {
    pub fn new_system(text: &str) -> Self {
        LLMMessage::System(text.to_string())
    }

    pub fn new_assistant(text: &str) -> Self {
        LLMMessage::Assistant(text.to_string())
    }

    pub fn new_user(text: &str) -> Self {
        LLMMessage::User(vec![LLMUserMessage::Text(text.to_string())])
    }

    pub fn new_user_with_image(text: &str, image_url: &str) -> Self {
        LLMMessage::User(vec![
            LLMUserMessage::Text(text.to_string()),
            LLMUserMessage::ImageUrl(image_url.to_string()),
        ])
    }

    pub fn new_user_with_images(text: &str, image_urls: &Vec<String>) -> Self {
        let mut messages: Vec<LLMUserMessage> = image_urls
            .iter()
            .map(|url| LLMUserMessage::ImageUrl(url.to_owned()))
            .collect();
        messages.insert(0, LLMUserMessage::Text(text.to_string()));
        LLMMessage::User(messages)
    }
}

#[derive(Debug, Clone)]
pub struct LLMInferenceParams {
    temperature: f64,
    seed: Option<u64>,
    top_p: Option<f64>,
    top_k: Option<usize>,
    max_tokens: Option<usize>,
    repeat_penalty: f32,
    repeat_last_n: usize,
}

impl Default for LLMInferenceParams {
    fn default() -> Self {
        Self {
            temperature: 0.8,
            seed: None,
            top_p: Some(0.9),
            top_k: Some(40),
            max_tokens: Some(512),
            repeat_penalty: 1.1,
            repeat_last_n: 64,
        }
    }
}

pub(crate) trait LLMModel {
    fn get_completion(
        &self,
        history: &[LLMMessage],
        params: LLMInferenceParams,
    ) -> impl std::future::Future<Output = anyhow::Result<LLMOutput>> + Send;
}

pub enum LLM {
    OpenAI(openai::OpenAI),
    Qwen2(qwen2::Qwen2),
    Qllama(qllama::Qllama),
}

impl Model for LLM {
    type Item = LLMInput;
    type Output = LLMOutput;

    fn batch_size_limit(&self) -> usize {
        1
    }

    async fn process(
        &mut self,
        items: Vec<Self::Item>,
    ) -> anyhow::Result<Vec<anyhow::Result<Self::Output>>> {
        let mut results = vec![];

        for item in items {
            let res = match self {
                LLM::OpenAI(model) => model.get_completion(&item.0, item.1).await,
                LLM::Qwen2(model) => model.get_completion(&item.0, item.1).await,
                LLM::Qllama(model) => model.get_completion(&item.0, item.1).await,
            };
            results.push(res);
        }

        Ok(results)
    }
}

#[cfg(test)]
mod test {
    use crate::llm::{
        openai::OpenAI, qllama::Qllama, qwen2::Qwen2, LLMInferenceParams, LLMMessage, LLMModel,
    };

    use super::LLMUserMessage;

    #[test_log::test(tokio::test)]
    async fn test_openai() {
        let client = OpenAI::new("http://localhost:11434/v1", "", "qwen2:7b-instruct-q4_0")
            .expect("failed to create client");

        let mut result = client
            .get_completion(
                &[LLMMessage::User(vec![LLMUserMessage::Text(
                    "Who are you?".into(),
                )])],
                super::LLMInferenceParams::default(),
            )
            .await
            .expect("");

        tracing::info!("result: {:?}", result.to_string().await);
    }

    #[test_log::test(tokio::test)]
    async fn test_qwen2() {
        let current_dir = std::env::current_dir().unwrap();
        let resource_dir = current_dir.join("../../apps/desktop/src-tauri/resources");
        let model = Qwen2::load(
            resource_dir.join("qwen2/qwen2-7b-instruct-q4_0.gguf"),
            resource_dir.join("qwen2/tokenizer.json"),
            "metal",
        )
        .expect("failed to load model");

        let mut result = model
            .get_completion(
                &[LLMMessage::User(vec![LLMUserMessage::Text(
                    "Who are you?".into(),
                )])],
                LLMInferenceParams::default(),
            )
            .await
            .expect("");

        tracing::info!("result: {:?}", result.to_string().await);
    }

    #[test_log::test(tokio::test)]
    async fn test_qllama() {
        let current_dir = std::env::current_dir().unwrap();
        let resource_dir = current_dir.join("../../apps/desktop/src-tauri/resources");
        let model = Qllama::load(
            resource_dir.join("llava-phi3-mini/llava-phi-3-mini-int4.gguf"),
            resource_dir.join("llava-phi3-mini/tokenizer.json"),
            "metal",
        )
        .expect("failed to load model");

        let messages = [
            LLMMessage::new_system("You are a bot."),
            LLMMessage::new_user("Hello."),
            // LLMMessage::User(vec![LLMUserMessage::Text("Who are you?".into())]),
        ];
        // let param = LLMInferenceParams {
        //     seed: Some(299792458),
        //     temperature: 0.7,
        //     top_p: None,
        //     top_k: None,
        //     ..Default::default()
        // };
        let param = LLMInferenceParams::default();
        let mut result = model.get_completion(&messages, param).await.expect("");

        tracing::info!("result: {:?}", result.to_string().await);
    }
}
