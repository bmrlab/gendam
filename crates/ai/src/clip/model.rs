#[derive(Clone)]
pub enum CLIPModel {
    MViTB32,
}

impl CLIPModel {
    pub fn model_uri(&self) -> (std::path::PathBuf, std::path::PathBuf, std::path::PathBuf) {
        match self {
            CLIPModel::MViTB32 => {
                let model_uri = std::path::Path::new("CLIP-ViT-B-32-multilingual-v1");

                (
                    model_uri.join("visual_quantize.onnx"),
                    model_uri.join("textual_quantize.onnx"),
                    model_uri.join("tokenizer.json"),
                )
            }
        }
    }

    pub fn dim(&self) -> usize {
        match self {
            CLIPModel::MViTB32 => 512,
        }
    }
}
