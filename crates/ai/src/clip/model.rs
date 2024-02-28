#[derive(Clone)]
pub enum CLIPModel {
    ViTB32,
    ViTL14,
}

impl CLIPModel {
    pub fn model_uri(&self) -> (std::path::PathBuf, std::path::PathBuf, std::path::PathBuf) {
        let model_uri = match self {
            CLIPModel::ViTB32 => std::path::Path::new("CLIP-ViT-B-32-laion2B-s34B-b79K"),
            CLIPModel::ViTL14 => todo!("add model info for ViT-L/14"),
        };

        (
            model_uri.join("visual.onnx"),
            model_uri.join("textual.onnx"),
            model_uri.join("tokenizer.json"),
        )
    }

    pub fn dim(&self) -> usize {
        match self {
            CLIPModel::ViTB32 => 512,
            CLIPModel::ViTL14 => 768,
        }
    }
}
