use ai::{
    blip::BLIP,
    clip::{CLIPModel, CLIP},
    text_embedding::OrtTextEmbedding,
    whisper::Whisper,
    AIModelLoader, AsAudioTranscriptModel, AsImageCaptionModel, AsMultiModalEmbeddingModel,
    AsTextEmbeddingModel, AudioTranscriptInput, AudioTranscriptOutput, ImageCaptionInput,
    ImageCaptionOutput, MultiModalEmbeddingInput, MultiModalEmbeddingOutput, TextEmbeddingInput,
    TextEmbeddingOutput,
};
use std::{fmt, path::PathBuf, time::Duration};

pub struct AIHandler {
    pub multi_modal_embedding: Box<dyn AsMultiModalEmbeddingModel + Send + Sync>,
    pub image_caption: Box<dyn AsImageCaptionModel + Send + Sync>,
    pub audio_transcript: Box<dyn AsAudioTranscriptModel + Send + Sync>,
    pub text_embedding: Box<dyn AsTextEmbeddingModel + Send + Sync>,
}

impl Clone for AIHandler {
    fn clone(&self) -> Self {
        Self {
            multi_modal_embedding: Box::new(AIModelLoader::<
                MultiModalEmbeddingInput,
                MultiModalEmbeddingOutput,
            > {
                tx: self.multi_modal_embedding.get_inputs_embedding_tx(),
            }),
            image_caption: Box::new(AIModelLoader::<ImageCaptionInput, ImageCaptionOutput> {
                tx: self.image_caption.get_images_caption_tx(),
            }),
            audio_transcript: Box::new(AIModelLoader::<
                AudioTranscriptInput,
                AudioTranscriptOutput,
            > {
                tx: self.audio_transcript.get_audio_transcript_tx(),
            }),
            text_embedding: Box::new(AIModelLoader::<TextEmbeddingInput, TextEmbeddingOutput> {
                tx: self.text_embedding.get_texts_embedding_tx(),
            }),
        }
    }
}

impl fmt::Debug for AIHandler {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("AIHandler").finish()
    }
}

pub fn init_ai_handlers(resources_dir: PathBuf) -> anyhow::Result<AIHandler> {
    let offload_duration = Duration::from_secs(30);

    let resources_dir_clone = resources_dir.clone();
    let image_caption_handler = AIModelLoader::<ImageCaptionInput, ImageCaptionOutput>::new(
        move || {
            let resources_dir_clone_clone = resources_dir_clone.clone();
            async move { BLIP::new(resources_dir_clone_clone, ai::blip::BLIPModel::Base).await }
        },
        Some(offload_duration.clone()),
    )?;

    let resources_dir_clone = resources_dir.clone();
    let multi_modal_embedding_handler =
        AIModelLoader::<MultiModalEmbeddingInput, MultiModalEmbeddingOutput>::new(
            move || {
                let resources_dir_clone_clone = resources_dir_clone.clone();
                async move { CLIP::new(resources_dir_clone_clone, CLIPModel::MViTB32).await }
            },
            Some(offload_duration.clone()),
        )?;

    let resources_dir_clone = resources_dir.clone();
    let audio_transcript_handler =
        AIModelLoader::<AudioTranscriptInput, AudioTranscriptOutput>::new(
            move || {
                let resources_dir_clone_clone = resources_dir_clone.clone();
                async move { Whisper::new(resources_dir_clone_clone).await }
            },
            Some(offload_duration.clone()),
        )?;

    let resources_dir_clone = resources_dir.clone();
    let text_embedding_handler = AIModelLoader::<TextEmbeddingInput, TextEmbeddingOutput>::new(
        move || {
            let resources_dir_clone_clone = resources_dir_clone.clone();
            async move { OrtTextEmbedding::new(resources_dir_clone_clone).await }
        },
        Some(offload_duration.clone()),
    )?;

    Ok(AIHandler {
        multi_modal_embedding: Box::new(multi_modal_embedding_handler),
        image_caption: Box::new(image_caption_handler),
        audio_transcript: Box::new(audio_transcript_handler),
        text_embedding: Box::new(text_embedding_handler),
    })
}
