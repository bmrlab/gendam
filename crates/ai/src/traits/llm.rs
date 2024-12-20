use super::{ImageCaptionInput, ImageCaptionModel};
use crate::{
    llm::{LLMInferenceParams, LLMMessage},
    AIModel,
};
use base64::Engine;
use futures::{Stream, StreamExt};
use std::{io::Cursor, pin::Pin};

pub type LLMInput = (Vec<LLMMessage>, LLMInferenceParams);
type LLMOutputInner = Pin<Box<dyn Stream<Item = anyhow::Result<Option<String>>> + Send + Sync>>;
pub type LLMModel = AIModel<LLMInput, LLMOutput>;

pub struct LLMOutput {
    inner: LLMOutputInner,
}

impl std::fmt::Debug for LLMOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LLMOutput")
            .field("inner", &"<stream>")
            .finish()
    }
}

impl Stream for LLMOutput {
    type Item = anyhow::Result<Option<String>>;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        self.inner.poll_next_unpin(cx)
    }
}

impl LLMOutput {
    pub fn new(inner: LLMOutputInner) -> Self {
        Self { inner }
    }

    pub async fn next(&mut self) -> Option<anyhow::Result<Option<String>>> {
        self.inner.next().await
    }

    pub async fn to_string(&mut self) -> anyhow::Result<String> {
        let mut output = String::new();
        // LLMOutput 是 LLMModel::get_completion 返回的结果
        // 这里会消费 async_stream::stream! 生成的流并且取 yield 的结果
        while let Some(item) = self.next().await {
            let item = item?;
            if let Some(item) = item {
                output.push_str(&item);
            }
        }
        Ok(output)
    }
}

impl LLMModel {
    /// This function takes a prompt string and returns an `ImageCaptionModel` that can be used
    /// to generate captions for images. This method also defines how the input image is processed.
    ///
    /// # Arguments
    /// * `prompt` - A string slice that contains the prompt to be used for image captioning.
    ///              This parameter is necessary because different LLM models may require different
    ///              prompts to effectively generate image captions. There isn't a one-size-fits-all
    ///              default prompt that works optimally for all LLM models.
    ///
    /// # Returns
    /// An `ImageCaptionModel` that can be used to generate captions for images.
    pub fn create_image_caption_ref(self) -> ImageCaptionModel {
        self.create_reference(
            move |v: ImageCaptionInput| {
                let prompt = v.prompt.unwrap_or("Please describe the image.".to_string());

                async move {
                    let result: Result<_, _> = {
                        let mut image_urls: Vec<String> = vec![];
                        for image_file_path in v.image_file_paths {
                            let image = image::ImageReader::open(image_file_path)?
                                .with_guessed_format()?
                                .decode()?;
                            let mut buf = Vec::new();
                            {
                                let mut cursor = Cursor::new(&mut buf);
                                let _ = image.write_to(&mut cursor, image::ImageFormat::Png);
                            }
                            let base64 = base64::engine::general_purpose::STANDARD.encode(&buf);
                            image_urls.push(format!("data:image/png;base64,{}", base64));
                        }
                        let params = LLMInferenceParams::default();
                        let messages: Vec<LLMMessage> = vec![
                            // LLMMessage::new_system(),
                            LLMMessage::new_user_with_images(prompt.clone().as_str(), &image_urls),
                        ];
                        Ok((messages, params))
                    };
                    result
                }
            },
            |mut v| async move {
                // 这个 convert_output 方法在 AIModel::create_reference 里调用,
                // LLMOutput 是一个异步闭包，调用 convert_output 的时候才会执行 to_string() 并获取 LLM 服务返回的结果
                v.to_string().await
            },
        )
    }
}

#[cfg(test)]
mod test {
    use std::{path::PathBuf, time::Duration};

    use crate::{
        llm::{openai::OpenAI, LLMInferenceParams, LLMMessage, LLM},
        AIModel, ImageCaptionInput,
    };

    #[test_log::test(tokio::test)]
    async fn test_llm_to_image_caption() {
        let llm = AIModel::new(
            "ollama-llava-phi3-mini".into(),
            move || async move {
                Ok(LLM::OpenAI(
                    OpenAI::new("http://localhost:11434/v1", "", "llava-phi3:3.8b-mini-q4_0")
                        .expect(""),
                ))
            },
            Some(Duration::from_secs(120)),
        )
        .expect("");

        let mut output = llm
            .process_single((
                vec![LLMMessage::new_user("who are you")],
                LLMInferenceParams::default(),
            ))
            .await
            .expect("");
        let output = output.to_string().await;
        tracing::info!("output: {:?}", output);

        let image_caption = llm.create_image_caption_ref();

        let result = image_caption
            .process_single(ImageCaptionInput {
                image_file_paths: vec![PathBuf::from("/Users/zhuo/Pictures/avatar.JPG")],
                prompt: None,
            })
            .await;

        tracing::info!("result: {:?}", result);
    }
}
