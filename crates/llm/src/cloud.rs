use async_trait::async_trait;
use crate::{Chat, LLMImageContent, LLMMessage, LLMParams};

#[allow(dead_code)]
struct CloudModel {
    api_endpoint: String,
    api_secret: Option<String>,
    is_multimodal: bool,
}

impl CloudModel {
    #[allow(dead_code)]
    pub fn new(api_endpoint: &str, api_secret: Option<&str>, is_multimodal: bool) -> Self {
        Self {
            api_endpoint: api_endpoint.into(),
            api_secret: api_secret.map(|v| v.into()),
            is_multimodal,
        }
    }
}

#[async_trait]
impl Chat for CloudModel {
    #[allow(unused_variables)]
    async fn get_completion(
        &self,
        history: Vec<LLMMessage>,
        images: Option<Vec<LLMImageContent>>,
        params: Option<LLMParams>,
    ) -> anyhow::Result<String> {
        todo!()
    }

    fn is_multimodal(&self) -> bool {
        self.is_multimodal
    }
}
