pub(crate) mod quantized_llama; // pub 给 llm 模型用

mod quantized_llava_phi3;
pub use quantized_llava_phi3::LLaVAPhi3Mini; // 只 pub 这个模型

use crate::traits::{ImageCaptionInput, ImageCaptionOutput, Model};

impl Model for LLaVAPhi3Mini {
    type Item = ImageCaptionInput;
    type Output = ImageCaptionOutput;

    fn batch_size_limit(&self) -> usize {
        1
    }

    async fn process(
        &mut self,
        items: Vec<Self::Item>,
    ) -> anyhow::Result<Vec<anyhow::Result<Self::Output>>> {
        if items.len() > self.batch_size_limit() {
            anyhow::bail!("too many items");
        }
        let mut results: Vec<Result<String, anyhow::Error>> = vec![];
        for item in items {
            let res = self.get_image_caption(item.image_file_paths, item.prompt);
            results.push(res);
        }
        Ok(results)
    }
}
