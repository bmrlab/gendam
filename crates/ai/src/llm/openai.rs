use super::LLMModel;
use crate::{
    llm::{LLMMessage, LLMUserMessage},
    LLMOutput,
};
use futures::StreamExt;
use reqwest::{
    self,
    header::{HeaderMap, AUTHORIZATION},
    Url,
};
use reqwest_eventsource::{Event, EventSource};
use serde::{Deserialize, Serialize};
use serde_json::{json, Deserializer, Value};
use std::str::FromStr;
use tokio::sync::mpsc;

#[allow(dead_code)]
pub struct OpenAI {
    base_url: String,
    model: String,
    headers: HeaderMap,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OpenAIResponseChoiceDelta {
    role: Option<String>,
    content: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OpenAIResponseChoice {
    index: Option<usize>,
    delta: Option<OpenAIResponseChoiceDelta>,
    finish_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OpenAIResponseChunk {
    id: Option<String>,
    object: Option<String>,
    created: Option<u64>,
    model: Option<String>,
    system_fingerprint: Option<String>,
    choices: Vec<OpenAIResponseChoice>,
}

impl LLMModel for OpenAI {
    async fn get_completion(
        &self,
        history: &[super::LLMMessage],
        params: super::LLMInferenceParams,
    ) -> anyhow::Result<LLMOutput> {
        let url = Url::parse(&self.base_url)?;
        let query = url.query();
        let mut url = url.join("chat/completions")?;
        url.set_query(query);

        let (tx, mut rx) = mpsc::channel(512);

        tracing::debug!("openai url: {:?}", url);

        let headers = self.headers.clone();
        let model = self.model.clone();
        let messages = history
            .iter()
            .map(|v| {
                let (role, message) = match v {
                    LLMMessage::System(v) => ("system", serde_json::to_value(v)),
                    LLMMessage::User(v) => (
                        "user",
                        if v.len() == 1 && matches!(v[0], LLMUserMessage::Text(_)) {
                            let text = match &v[0] {
                                LLMUserMessage::Text(text) => text,
                                _ => unreachable!(),
                            };

                            serde_json::to_value(text)
                        } else {
                            serde_json::to_value(
                                v.iter()
                                    .map(|t| match t {
                                        LLMUserMessage::ImageUrl(image_url) => {
                                            json!({"type": "image_url", "image_url": image_url})
                                        }
                                        LLMUserMessage::Text(text) => {
                                            json!({"type": "text", "text": text})
                                        }
                                    })
                                    .collect::<Vec<_>>(),
                            )
                        },
                    ),
                    LLMMessage::Assistant(v) => ("assistant", serde_json::to_value(v)),
                };

                json!({
                    "role": role,
                    "content": message.expect("message should be valid json")
                })
            })
            .collect::<Vec<Value>>();

        tokio::spawn(async move {
            let client = reqwest::Client::new().post(url).headers(headers).body(
                json!({
                    "model": &model,
                    "messages": messages,
                    "stream": true,
                    "temperature": params.temperature,
                    "seed": params.seed,
                    "top_p": params.top_p,
                    "max_tokens": params.max_tokens
                })
                .to_string(),
            );

            let mut es = EventSource::new(client).expect("event source created");
            let mut buffer = String::new(); // a buffer to contain possible incomplete message

            while let Some(event) = es.next().await {
                match event {
                    Ok(Event::Open) => {
                        tracing::debug!("stream opened");
                    }
                    Ok(Event::Message(message)) => {
                        // sometimes message.data is not a complete JSON value, especially when using AzureOpenAI API
                        // so here use a buffer to contain them and try to extract json from buffer
                        buffer.push_str(&message.data);

                        let mut deserialize_error = None;
                        let byte_offset;
                        {
                            let mut stream_deserializer =
                                Deserializer::from_str(&buffer).into_iter::<OpenAIResponseChunk>();

                            while let Some(result) = stream_deserializer.next() {
                                match result {
                                    Ok(response) => {
                                        for choice in &response.choices {
                                            if let Some(OpenAIResponseChoiceDelta {
                                                content: Some(response_content),
                                                ..
                                            }) = &choice.delta
                                            {
                                                // here role must be assistant, just ignore

                                                if let Err(e) = tx
                                                    .send(Ok(Some(response_content.clone())))
                                                    .await
                                                {
                                                    tracing::error!(
                                                        "failed to send response: {}",
                                                        e
                                                    );
                                                }

                                                if let Some(finish_reason) = &choice.finish_reason {
                                                    tracing::debug!(
                                                        "LLM finish reason: {:?}",
                                                        finish_reason
                                                    );

                                                    if let Err(e) = tx.send(Ok(None)).await {
                                                        tracing::error!(
                                                            "failed to send finish reason: {}",
                                                            e
                                                        );
                                                    }

                                                    // to break or not to break, both work
                                                    // break;
                                                }
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        deserialize_error = Some(e);
                                        break;
                                    }
                                }
                            }

                            byte_offset = stream_deserializer.byte_offset();
                        }

                        // Remove the parsed JSON part from the buffer
                        buffer.drain(..byte_offset);

                        if let Some(err) = deserialize_error {
                            if !err.is_eof() {
                                // this is a real error
                                tracing::error!("failed to parse response: {}", &buffer);
                                buffer.clear();
                            }
                        }
                    }
                    Err(reqwest_eventsource::Error::StreamEnded) => {
                        tracing::debug!("stream ended");
                        break;
                    }
                    Err(e) => {
                        tracing::error!("failed to handle event source: {}", e);
                        break;
                    }
                }
            }
        });

        let stream = async_stream::stream! {
            while let Some(result) = rx.recv().await {
                yield result;
            }
        };

        Ok(LLMOutput::new(Box::pin(stream)))
    }
}

impl OpenAI {
    /// Create a new OpenAI compatible chat completion client.
    ///
    /// TODO
    /// - it is better to pass model when inference
    pub fn new(base_url: &str, api_key: &str, model: &str) -> anyhow::Result<Self> {
        let base_url = if base_url.ends_with("/") {
            base_url.to_string()
        } else {
            format!("{}/", base_url)
        };

        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, format!("Bearer {}", api_key).parse()?);

        Ok(Self {
            base_url,
            model: model.to_string(),
            headers,
        })
    }

    pub fn new_azure(
        azure_endpoint: &str,
        api_key: &str,
        deployment_name: &str,
        api_version: &str,
    ) -> anyhow::Result<Self> {
        let base_url = Url::from_str(azure_endpoint)?;
        let mut base_url = base_url.join(&format!("openai/deployments/{}/", deployment_name))?;
        base_url.set_query(Some(&format!("api-version={}", api_version)));

        let mut headers = HeaderMap::new();
        headers.insert("api-key", api_key.parse()?);

        Ok(Self {
            base_url: base_url.to_string(),
            model: deployment_name.to_string(),
            headers,
        })
    }
}
