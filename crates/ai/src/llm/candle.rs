use super::{native::LocalLLMModel, LLMInferenceParams, LLMModel};
use candle_core::{Device, Tensor};
use candle_transformers::generation::{LogitsProcessor, Sampling};
use tokenizers::Tokenizer;
use tokio::sync::mpsc::Sender;

/// This is a wrapper around a tokenizer to ensure that tokens can be returned to the user in a
/// streaming way rather than having to wait for the full decoding.
pub struct TokenOutputStream {
    tokenizer: tokenizers::Tokenizer,
    tokens: Vec<u32>,
    prev_index: usize,
    current_index: usize,
}

impl TokenOutputStream {
    pub fn new(tokenizer: tokenizers::Tokenizer) -> Self {
        Self {
            tokenizer,
            tokens: Vec::new(),
            prev_index: 0,
            current_index: 0,
        }
    }

    pub fn into_inner(self) -> tokenizers::Tokenizer {
        self.tokenizer
    }

    fn decode(&self, tokens: &[u32]) -> candle_core::Result<String> {
        match self.tokenizer.decode(tokens, true) {
            Ok(str) => Ok(str),
            Err(err) => candle_core::bail!("cannot decode: {err}"),
        }
    }

    // https://github.com/huggingface/text-generation-inference/blob/5ba53d44a18983a4de32d122f4cb46f4a17d9ef6/server/text_generation_server/models/model.py#L68
    pub fn next_token(&mut self, token: u32) -> candle_core::Result<Option<String>> {
        let prev_text = if self.tokens.is_empty() {
            String::new()
        } else {
            let tokens = &self.tokens[self.prev_index..self.current_index];
            self.decode(tokens)?
        };
        self.tokens.push(token);
        let text = self.decode(&self.tokens[self.prev_index..])?;
        if text.len() > prev_text.len() && text.chars().last().unwrap().is_alphanumeric() {
            let text = text.split_at(prev_text.len());
            self.prev_index = self.current_index;
            self.current_index = self.tokens.len();
            Ok(Some(text.1.to_string()))
        } else {
            Ok(None)
        }
    }

    pub fn decode_rest(&self) -> candle_core::Result<Option<String>> {
        let prev_text = if self.tokens.is_empty() {
            String::new()
        } else {
            let tokens = &self.tokens[self.prev_index..self.current_index];
            self.decode(tokens)?
        };
        let text = self.decode(&self.tokens[self.prev_index..])?;
        if text.len() > prev_text.len() {
            let text = text.split_at(prev_text.len());
            Ok(Some(text.1.to_string()))
        } else {
            Ok(None)
        }
    }

    pub fn decode_all(&self) -> candle_core::Result<String> {
        self.decode(&self.tokens)
    }

    pub fn get_token(&self, token_s: &str) -> Option<u32> {
        self.tokenizer.get_vocab(true).get(token_s).copied()
    }

    pub fn tokenizer(&self) -> &tokenizers::Tokenizer {
        &self.tokenizer
    }

    pub fn clear(&mut self) {
        self.tokens.clear();
        self.prev_index = 0;
        self.current_index = 0;
    }
}

pub(crate) trait CandleLLMModel: LLMModel + LocalLLMModel {
    async fn forward(
        &self,
        input: &str,
        params: LLMInferenceParams,
        tx: Option<Sender<anyhow::Result<Option<String>>>>,
    ) -> anyhow::Result<String> {
        let mut tos = TokenOutputStream::new(self.tokenizers());

        let prompt_str = input.to_string();
        let tokens = tos
            .tokenizer()
            .encode(prompt_str, true)
            .map_err(|err| anyhow::anyhow!(err))?;
        let prompt_tokens = [tokens.get_ids()].concat();

        let device = self.device();

        let mut all_tokens = vec![];

        let mut logits_processor = {
            let temperature = params.temperature;
            let sampling = if temperature <= 0. {
                Sampling::ArgMax
            } else {
                match (params.top_k, params.top_p) {
                    (None, None) => Sampling::All { temperature },
                    (Some(k), None) => Sampling::TopK { k, temperature },
                    (None, Some(p)) => Sampling::TopP { p, temperature },
                    (Some(k), Some(p)) => Sampling::TopKThenTopP { k, p, temperature },
                }
            };
            LogitsProcessor::from_sampling(params.seed.unwrap_or(rand::random()), sampling)
        };

        #[cfg(debug_assertions)]
        {
            println!("");
        }
        let mut next_token = {
            let input = Tensor::new(prompt_tokens.as_slice(), &device)?.unsqueeze(0)?;

            let logits = self.next_token_logits(&input, 0)?;
            let logits = logits.squeeze(0)?;
            let next_token = logits_processor.sample(&logits)?;
            all_tokens.push(next_token);

            if let Some(token) = tos.next_token(next_token)? {
                #[cfg(debug_assertions)]
                {
                    use std::io::Write;
                    print!("{}", token);
                    std::io::stdout().flush()?;
                }
                if let Some(tx) = tx.clone() {
                    tx.send(Ok(Some(token))).await?;
                }
            }
            // if let Some(tx) = tx.clone() {
            //     tx.send(Ok(Some(tos.decode(&[next_token])?))).await?;
            // }
            next_token
        };

        let eos_token = {
            let eos_token = self.end_of_turn();
            *tos.tokenizer().get_vocab(true).get(&eos_token).unwrap()
        };
        let mut index = 0;
        loop {
            let input = Tensor::new(&[next_token], &device)?.unsqueeze(0)?;

            let logits = self.next_token_logits(&input, prompt_tokens.len() + index)?;
            let logits = logits.squeeze(0)?;
            let logits = if params.repeat_penalty == 1. {
                logits
            } else {
                let start_at = all_tokens.len().saturating_sub(params.repeat_last_n);
                candle_transformers::utils::apply_repeat_penalty(
                    &logits,
                    params.repeat_penalty,
                    &all_tokens[start_at..],
                )?
            };

            next_token = logits_processor.sample(&logits)?;
            all_tokens.push(next_token);
            index += 1;

            if next_token == eos_token {
                if let Some(tx) = tx.clone() {
                    tx.send(Ok(None)).await?;
                }
                break;
            }

            if let Some(token) = tos.next_token(next_token)? {
                #[cfg(debug_assertions)]
                {
                    use std::io::Write;
                    print!("{}", token);
                    std::io::stdout().flush()?;
                }
                if let Some(tx) = tx.clone() {
                    tx.send(Ok(Some(token))).await?;
                }
            }
            // if let Some(tx) = tx.clone() {
            //     tx.send(Ok(Some(tos.decode(&[next_token])?))).await?;
            // }

            if let Some(max_tokens) = params.max_tokens {
                if index >= max_tokens {
                    break;
                }
            }
        }
        #[cfg(debug_assertions)]
        {
            println!("\n");
        }

        tos.decode_all().map_err(|err| anyhow::anyhow!(err))
        // tos.decode(&all_tokens).map_err(|err| anyhow::anyhow!(err))
    }

    fn next_token_logits(&self, input: &Tensor, index_pos: usize) -> anyhow::Result<Tensor>;
    fn tokenizers(&self) -> Tokenizer;
    fn device(&self) -> Device;
}
