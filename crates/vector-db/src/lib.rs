#[cfg(feature = "faiss")]
pub mod faiss;

#[cfg(feature = "faiss")]
pub use faiss::*;

const DEFAULT_VISION_COLLECTION_NAME: &str = "gendam-vision";
const DEFAULT_LANGUAGE_COLLECTION_NAME: &str = "gendam-language";

pub fn get_language_collection_name(model_id: &str) -> String {
    format!("{}-{}", DEFAULT_LANGUAGE_COLLECTION_NAME, model_id)
}

pub fn get_vision_collection_name(model_id: &str) -> String {
    format!("{}-{}", DEFAULT_VISION_COLLECTION_NAME, model_id)
}
