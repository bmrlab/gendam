use super::LLMMessage;

pub trait LocalLLMModel {
    fn start_of_turn(&self) -> String;
    fn end_of_turn(&self) -> String;
    fn system_name(&self) -> String;
    fn user_name(&self) -> String;
    fn assistant_name(&self) -> String;

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
            "{}{}{}\n",
            prompt,
            self.start_of_turn(),
            self.assistant_name()
        )
    }
}
