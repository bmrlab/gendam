use crate::llm::LLMUserMessage;

use super::LLMMessage;

pub trait LocalLLMModel {
    fn start_of_turn(&self) -> String;
    fn end_of_turn(&self) -> String;
    fn system_name(&self) -> String;
    fn user_name(&self) -> String;
    fn assistant_name(&self) -> String;

    /// Image in User prompt is not supported yet.
    fn with_chat_template(&self, history: &[LLMMessage]) -> String {
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
                        v.iter()
                            .filter_map(|t| match t {
                                LLMUserMessage::Text(content) => Some(content.clone()),
                                _ => None, // FIXME currently, image content is ignored for native model
                            })
                            .collect::<Vec<_>>()
                            .join("\n"),
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
            "{}{}{}\n",
            prompt,
            self.start_of_turn(),
            self.assistant_name()
        )
    }
}
