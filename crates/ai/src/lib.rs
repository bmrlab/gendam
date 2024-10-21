pub mod blip;
pub mod clip;
pub mod llava_phi3_mini;
pub mod llm;
mod loader;
pub mod moondream;
pub(crate) mod ort;
pub mod text_embedding;
mod traits;
pub mod utils;
pub mod whisper;
pub mod yolo;
pub use tokenizers;
use tokio::sync::oneshot;
pub use traits::*;

pub type HandlerPayload<TItem, TOutput> = (
    Vec<TItem>,
    oneshot::Sender<anyhow::Result<Vec<anyhow::Result<TOutput>>>>,
);
