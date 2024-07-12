use crate::{llm::{LLMInferenceParams, LLMMessage}, AIModelLoader, AIModelTx};

pub trait AsLLM: Send + Sync {
    fn get_llm_tx(&self) -> AIModelTx<(Vec<LLMMessage>, LLMInferenceParams), String>;
}

impl AsLLM for AIModelLoader<(Vec<LLMMessage>, LLMInferenceParams), String> {
    fn get_llm_tx(&self) -> AIModelTx<(Vec<LLMMessage>, LLMInferenceParams), String> {
        self.tx.clone()
    }
}
