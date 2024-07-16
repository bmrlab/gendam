use std::{
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

use futures::Stream;
use tokio::{sync::mpsc::Sender, time::sleep};

use crate::Model;

pub mod candle;
pub mod native;
pub mod openai;
pub mod qwen2;

#[derive(Debug, Clone)]
pub enum LLMMessage {
    System(String),
    User(String),
    Assistant(String),
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
        &mut self,
        history: &[LLMMessage],
        params: LLMInferenceParams,
        tx: Sender<anyhow::Result<Option<String>>>,
    ) -> impl std::future::Future<Output = anyhow::Result<String>> + Send;
}

pub enum LLM {
    OpenAI(openai::OpenAI),
    Qwen2(qwen2::Qwen2),
}

impl Model for LLM {
    type Item = (
        Vec<LLMMessage>,
        LLMInferenceParams,
        Sender<anyhow::Result<Option<String>>>,
    );
    type Output = String;

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
                LLM::OpenAI(model) => model.get_completion(&item.0, item.1, item.2).await,
                LLM::Qwen2(model) => model.get_completion(&item.0, item.1, item.2).await,
            };
            results.push(res);
        }

        Ok(results)
    }
}

pub struct LLMOutput {
    count: usize,
}

impl Stream for LLMOutput {
    type Item = String;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        if this.count < 10 {
            this.count += 1;

            // Simulate an async operation using sleep
            let waker = cx.waker().clone();
            tokio::spawn(async move {
                sleep(Duration::from_millis(500)).await;
                waker.wake();
            });

            Poll::Pending
        } else {
            Poll::Ready(None)
        }
    }
}

// #[tokio::test]
// async fn test_qwen2() {
//     let mut model = qwen2::Qwen2::load(
//         "/Users/zhuo/Downloads/qwen2-7b-instruct-q4_0.gguf",
//         "/Users/zhuo/Downloads/tokenizer-qwen2-7b.json",
//         "metal",
//     )
//     .expect("failed to load model");

//     let response = model
//         .get_completion(
//             &[LLMMessage::User("who are you?".into())],
//             LLMInferenceParams::default(),
//         )
//         .await;

//     println!("response: {:?}", response);
// }
