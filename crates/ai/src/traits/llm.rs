use crate::{
    llm::{LLMInferenceParams, LLMMessage},
    AIModel,
};

pub type LLMModel = AIModel<(Vec<LLMMessage>, LLMInferenceParams), String>;
