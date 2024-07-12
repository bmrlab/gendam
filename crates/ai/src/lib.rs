pub mod blip;
pub mod clip;
pub mod llm;
mod loader;
pub mod moondream;
pub(crate) mod ort;
pub mod text_embedding;
mod traits;
pub mod utils;
pub mod whisper;
pub mod yolo;
pub use traits::*;

pub enum HandlerPayload<T> {
    BatchData(T),
    Shutdown,
}
