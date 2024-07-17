use crate::{
    llm::{LLMInferenceParams, LLMMessage},
    AIModel,
};
use futures::{Stream, StreamExt};
use std::pin::Pin;

pub type LLMInput = (Vec<LLMMessage>, LLMInferenceParams);
type LLMOutputInner = Pin<Box<dyn Stream<Item = anyhow::Result<Option<String>>> + Send + Sync>>;
pub type LLMModel = AIModel<LLMInput, LLMOutput>;

pub struct LLMOutput {
    inner: LLMOutputInner,
}

impl Stream for LLMOutput {
    type Item = anyhow::Result<Option<String>>;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        self.inner.poll_next_unpin(cx)
    }
}

impl LLMOutput {
    pub fn new(inner: LLMOutputInner) -> Self {
        Self { inner }
    }

    pub async fn next(&mut self) -> Option<anyhow::Result<Option<String>>> {
        self.inner.next().await
    }

    pub async fn to_string(&mut self) -> anyhow::Result<String> {
        let mut output = String::new();
        while let Some(item) = self.next().await {
            let item = item?;
            if let Some(item) = item {
                output.push_str(&item);
            }
        }
        Ok(output)
    }
}
