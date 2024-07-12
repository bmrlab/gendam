pub mod candle;
pub mod native;
pub mod openai;
pub mod qwen2;

pub enum LLMMessage {
    System(String),
    User(String),
    Assistant(String),
}

#[derive(Debug)]
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

pub trait LLMModel {
    fn get_completion(
        &mut self,
        history: &[LLMMessage],
        params: LLMInferenceParams,
    ) -> impl std::future::Future<Output = anyhow::Result<String>> + Send;

    fn get_completion_with_image(
        &mut self,
        history: &[LLMMessage],
        image_url: &str,
        params: LLMInferenceParams,
    ) -> impl std::future::Future<Output = anyhow::Result<String>> + Send;
}

#[tokio::test]
async fn test_qwen2() {
    let mut model = qwen2::Qwen2::load(
        "/Users/zhuo/Downloads/qwen2-7b-instruct-q4_0.gguf",
        "/Users/zhuo/Downloads/tokenizer-qwen2-7b.json",
        candle_core::Device::new_metal(0).expect("failed to init metal device"),
    )
    .expect("failed to load model");

    let response = model
        .get_completion(
            &[LLMMessage::User("who are you?".into())],
            LLMInferenceParams::default(),
        )
        .await;

    println!("response: {:?}", response);
}
