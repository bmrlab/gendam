use tokio::sync::mpsc::Sender;

use crate::{
    llm::{LLMInferenceParams, LLMMessage},
    AIModel,
};

pub type LLMInput = (
    Vec<LLMMessage>,
    LLMInferenceParams,
    Sender<anyhow::Result<Option<String>>>,
);
pub type LLMOutput = String;

pub type LLMModel = AIModel<LLMInput, LLMOutput>;
