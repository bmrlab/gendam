mod loader;
mod ort;
mod traits;

pub mod blip;
pub mod clip;
pub mod llava_phi3_mini;
pub mod llm;
pub mod moondream;
pub mod text_embedding;
pub mod utils;
pub mod whisper;
pub mod yolo;

pub use tokenizers;
pub use traits::*;

use tokio::sync::oneshot;

pub type HandlerPayload<TItem, TOutput> = (
    Vec<TItem>,
    oneshot::Sender<anyhow::Result<Vec<anyhow::Result<TOutput>>>>,
);
