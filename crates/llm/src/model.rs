use crate::LLMMessage;

#[derive(Debug, Clone, Copy)]
pub enum Model {
    Gemma2B,
    QWen0_5B,
}

impl Model {
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
        }
        .into()
    }
    pub fn start_of_turn(self) -> String {
        match self {
            Self::Gemma2B => "<start_of_turn>",
            Self::QWen0_5B => "<|im_start|>",
        }
        .into()
    }

    pub fn end_of_turn(self) -> String {
        match self {
            Self::Gemma2B => "<end_of_turn>",
            Self::QWen0_5B => "<|im_end|>",
        }
        .into()
    }

    pub fn system_name(self) -> String {
        match self {
            Self::Gemma2B => "system",
            Self::QWen0_5B => "system",
        }
        .into()
    }

    pub fn user_name(self) -> String {
        match self {
            Self::Gemma2B => "user",
            Self::QWen0_5B => "user",
        }
        .into()
    }

    pub fn assistant_name(self) -> String {
        match self {
            Self::Gemma2B => "model",
            Self::QWen0_5B => "assistant",
        }
        .into()
    }
}
