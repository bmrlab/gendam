use super::LLMMessage;

#[derive(Debug, Clone, Copy)]
pub enum LlamaCppModel {
    Gemma2B,
    QWen0_5B,
    LLaVaMistral,
}

impl LlamaCppModel {
    pub fn with_chat_template(self, history: Vec<LLMMessage>) -> String {
        let prompt = history
            .iter()
            .map(|v| match v {
                LLMMessage::System(v) => {
                    format!(
                        "{}{}\n{}{}",
                        self.start_of_turn(),
                        self.system_name(),
                        v,
                        self.end_of_turn()
                    )
                }
                LLMMessage::User(v) => {
                    format!(
                        "{}{}\n{}{}",
                        self.start_of_turn(),
                        self.user_name(),
                        v,
                        self.end_of_turn()
                    )
                }
                LLMMessage::Assistant(v) => {
                    format!(
                        "{}{}\n{}{}\n",
                        self.start_of_turn(),
                        self.assistant_name(),
                        v,
                        self.end_of_turn()
                    )
                }
            })
            .collect::<Vec<String>>()
            .join("");

        format!(
            "{}{}{}",
            prompt,
            self.start_of_turn(),
            self.assistant_name()
        )
    }

    pub fn model_uri(self) -> String {
        match self {
            Self::Gemma2B => "Gemma/2b.gguf",
            Self::QWen0_5B => "qwen/0.5b.gguf",
            Self::LLaVaMistral => "llava-v1.6-7b/ggml-mistral-q_4_k.gguf",
        }
        .into()
    }

    pub fn mmproj_uri(self) -> Option<String> {
        match self {
            Self::LLaVaMistral => Some("llava-v1.6-7b/mmproj-mistral7b-f16-q6_k.gguf"),
            _ => None,
        }
        .map(|v| v.into())
    }

    pub fn start_of_turn(self) -> String {
        match self {
            Self::Gemma2B => "<start_of_turn>",
            Self::QWen0_5B => "<|im_start|>",
            Self::LLaVaMistral => "<|im_start|>",
        }
        .into()
    }

    pub fn end_of_turn(self) -> String {
        match self {
            Self::Gemma2B => "<end_of_turn>",
            Self::QWen0_5B => "<|im_end|>",
            Self::LLaVaMistral => "<|im_end|>",
        }
        .into()
    }

    pub fn system_name(self) -> String {
        match self {
            _ => "system",
        }
        .into()
    }

    pub fn user_name(self) -> String {
        match self {
            _ => "user",
        }
        .into()
    }

    pub fn assistant_name(self) -> String {
        match self {
            Self::Gemma2B => "model",
            Self::QWen0_5B => "assistant",
            Self::LLaVaMistral => "assistant",
        }
        .into()
    }

    pub fn is_multi_modal(self) -> bool {
        match self {
            Self::LLaVaMistral => true,
            _ => false,
        }
    }
}
