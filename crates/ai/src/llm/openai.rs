use super::LLMModel;

#[allow(dead_code)]
pub struct OpenAI {
    base_url: String,
    api_key: String,
}

impl LLMModel for OpenAI {
    async fn get_completion(
        &mut self,
        _history: &[super::LLMMessage],
        _params: super::LLMInferenceParams,
    ) -> anyhow::Result<String> {
        todo!()
    }

    async fn get_completion_with_image(
        &mut self,
        _history: &[super::LLMMessage],
        _image_url: &str,
        _params: super::LLMInferenceParams,
    ) -> anyhow::Result<String> {
        todo!()
    }
}

impl OpenAI {
    #[allow(dead_code)]
    pub fn new(base_url: &str, api_key: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
            api_key: api_key.to_string(),
        }
    }
}
