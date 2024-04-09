use ai::{blip::BLIP, clip::CLIP, text_embedding::TextEmbedding, whisper::Whisper, BatchHandler};
use std::{path::PathBuf, time::Duration};

#[derive(Clone, Debug)]
pub struct AIHandler {
    pub clip: BatchHandler<CLIP>,
    pub blip: BatchHandler<BLIP>,
    pub whisper: BatchHandler<Whisper>,
    pub text_embedding: BatchHandler<TextEmbedding>,
}

pub fn init_ai_handlers(resources_dir: PathBuf) -> anyhow::Result<AIHandler> {
    let offload_duration = Duration::from_secs(30);

    let resources_dir_clone = resources_dir.clone();
    let blip_handler = BatchHandler::new(
        move || {
            let resources_dir_clone_clone = resources_dir_clone.clone();
            async move {
                ai::blip::BLIP::new(resources_dir_clone_clone, ai::blip::BLIPModel::Base).await
            }
        },
        Some(offload_duration.clone()),
    )?;

    let resources_dir_clone = resources_dir.clone();
    let clip_handler = BatchHandler::new(
        move || {
            let resources_dir_clone_clone = resources_dir_clone.clone();
            async move {
                ai::clip::CLIP::new(ai::clip::CLIPModel::ViTB32, resources_dir_clone_clone).await
            }
        },
        Some(offload_duration.clone()),
    )?;

    let resources_dir_clone = resources_dir.clone();
    let whisper_handler = BatchHandler::new(
        move || {
            let resources_dir_clone_clone = resources_dir_clone.clone();
            async move { Whisper::new(resources_dir_clone_clone).await }
        },
        Some(offload_duration.clone()),
    )?;

    let resources_dir_clone = resources_dir.clone();
    let text_embedding_handler = BatchHandler::new(
        move || {
            let resources_dir_clone_clone = resources_dir_clone.clone();
            async move { TextEmbedding::new(resources_dir_clone_clone).await }
        },
        Some(offload_duration.clone()),
    )?;

    Ok(AIHandler {
        clip: clip_handler,
        blip: blip_handler,
        whisper: whisper_handler,
        text_embedding: text_embedding_handler,
    })
}
