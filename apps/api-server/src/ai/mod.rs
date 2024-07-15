pub mod models;

use self::models::{get_model_info_by_id, ConcreteModelType};
use crate::{library::get_library_settings, CtxWithLibrary};
use ai::{
    blip::BLIP,
    clip::{CLIPModel, CLIP},
    llm::{qwen2::Qwen2, LLM},
    text_embedding::OrtTextEmbedding,
    whisper::Whisper,
    AIModel, AudioTranscriptModel, ImageCaptionModel, LLMModel, MultiModalEmbeddingModel,
    TextEmbeddingModel,
};
use anyhow::bail;
use serde_json::Value;
use std::{fmt, time::Duration};

pub struct AIHandler {
    pub multi_modal_embedding: MultiModalEmbeddingModel,
    pub image_caption: ImageCaptionModel,
    pub audio_transcript: AudioTranscriptModel,
    pub text_embedding: TextEmbeddingModel,
    pub llm: LLMModel,
}

impl Clone for AIHandler {
    fn clone(&self) -> Self {
        Self {
            multi_modal_embedding: self.multi_modal_embedding.clone(),
            image_caption: self.image_caption.clone(),
            audio_transcript: self.audio_transcript.clone(),
            text_embedding: self.text_embedding.clone(),
            llm: self.llm.clone(),
        }
    }
}

impl fmt::Debug for AIHandler {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("AIHandler").finish()
    }
}

fn get_str_from_params(params: &Value, name: &str) -> Result<String, rspc::Error> {
    match params[name].as_str() {
        Some(s) => Ok(s.into()),
        _ => Err(rspc::Error::new(
            rspc::ErrorCode::InternalServerError,
            format!("invalid {}", name),
        )),
    }
}

impl AIHandler {
    pub fn new(ctx: &dyn CtxWithLibrary) -> Result<Self, rspc::Error> {
        let multi_modal_embedding = Self::get_multi_modal_embedding(ctx)?;
        let text_embedding = Self::get_text_embedding(ctx, &multi_modal_embedding)?;

        Ok(Self {
            multi_modal_embedding,
            image_caption: Self::get_image_caption(ctx)?,
            audio_transcript: Self::get_audio_transcript(ctx)?,
            text_embedding,
            llm: Self::get_llm(ctx)?,
        })
    }

    fn get_image_caption(ctx: &dyn CtxWithLibrary) -> Result<ImageCaptionModel, rspc::Error> {
        let resources_dir = ctx.get_resources_dir().to_path_buf();
        let library = ctx.library()?;
        let settings = get_library_settings(&library.dir);

        let model = get_model_info_by_id(ctx, &settings.models.image_caption)?;
        let handler = AIModel::new(
            move || {
                let resources_dir_clone = resources_dir.clone();
                let model_clone = model.clone();
                async move {
                    match model_clone.model_type {
                        ConcreteModelType::BLIP => {
                            let params = model_clone.params;
                            let model_path = resources_dir_clone
                                .join(get_str_from_params(&params, "model_path")?);
                            let tokenizer_path = resources_dir_clone
                                .join(get_str_from_params(&params, "tokenizer_path")?);
                            let model_type = get_str_from_params(&params, "model_type")?;
                            let model_type = match model_type.as_str() {
                                "Large" => ai::blip::BLIPModel::Large,
                                _ => ai::blip::BLIPModel::Base,
                            };
                            BLIP::new(model_path, tokenizer_path, model_type).await
                        }
                        _ => {
                            bail!(
                                "unsupported model {} for image caption",
                                model_clone.model_type.as_ref()
                            )
                        }
                    }
                }
            },
            Some(Duration::from_secs(30)),
        )
        .map_err(|e| rspc::Error::new(rspc::ErrorCode::InternalServerError, e.to_string()))?;

        Ok(handler)
    }

    fn get_multi_modal_embedding(
        ctx: &dyn CtxWithLibrary,
    ) -> Result<MultiModalEmbeddingModel, rspc::Error> {
        let resources_dir = ctx.get_resources_dir().to_path_buf();
        let library = ctx.library()?;
        let settings = get_library_settings(&library.dir);

        let model = get_model_info_by_id(ctx, &settings.models.multi_modal_embedding)?;
        let handler = AIModel::new(
            move || {
                let resources_dir_clone = resources_dir.clone();
                let model_clone = model.clone();
                async move {
                    let params = model_clone.params;
                    match model_clone.model_type {
                        ConcreteModelType::CLIP => {
                            let image_model_path = resources_dir_clone
                                .join(get_str_from_params(&params, "image_model_path")?);

                            let text_model_path = resources_dir_clone
                                .join(get_str_from_params(&params, "text_model_path")?);
                            let text_tokenizer_vocab_path = resources_dir_clone
                                .join(get_str_from_params(&params, "text_tokenizer_vocab_path")?);
                            CLIP::new(
                                image_model_path,
                                text_model_path,
                                text_tokenizer_vocab_path,
                                CLIPModel::MViTB32,
                            )
                            .await
                        }
                        _ => {
                            bail!(
                                "unsupported model {} for multi modal embedding",
                                model_clone.model_type.as_ref()
                            )
                        }
                    }
                }
            },
            Some(Duration::from_secs(30)),
        )
        .map_err(|e| rspc::Error::new(rspc::ErrorCode::InternalServerError, e.to_string()))?;

        Ok(handler)
    }

    fn get_audio_transcript(ctx: &dyn CtxWithLibrary) -> Result<AudioTranscriptModel, rspc::Error> {
        let resources_dir = ctx.get_resources_dir().to_path_buf();
        let library = ctx.library()?;
        let settings = get_library_settings(&library.dir);

        let model = get_model_info_by_id(ctx, &settings.models.audio_transcript)?;
        let handler = AIModel::new(
            move || {
                let resources_dir_clone = resources_dir.clone();
                let model_clone = model.clone();
                async move {
                    let params = model_clone.params;
                    match model_clone.model_type {
                        ConcreteModelType::Whisper => {
                            let model_path = resources_dir_clone
                                .join(get_str_from_params(&params, "model_path")?);
                            Whisper::new(model_path).await
                        }
                        _ => {
                            bail!(
                                "unsupported model {} for multi modal embedding",
                                model_clone.model_type.as_ref()
                            )
                        }
                    }
                }
            },
            Some(Duration::from_secs(30)),
        )
        .map_err(|e| rspc::Error::new(rspc::ErrorCode::InternalServerError, e.to_string()))?;

        Ok(handler)
    }

    /// Get text embedding model.
    ///
    /// ⚠️ 因为 multi_modal_embedding_model 也能完成 text_embedding，所以这里也传入他，避免重复加载同样的模型
    fn get_text_embedding(
        ctx: &dyn CtxWithLibrary,
        multi_modal_handler: &MultiModalEmbeddingModel,
    ) -> Result<TextEmbeddingModel, rspc::Error> {
        let resources_dir = ctx.get_resources_dir().to_path_buf();
        let library = ctx.library()?;
        let settings = get_library_settings(&library.dir);

        if settings.models.text_embedding == settings.models.multi_modal_embedding {
            return Ok(multi_modal_handler.into());
        }

        let model = get_model_info_by_id(ctx, &settings.models.text_embedding)?;

        if model.model_type == ConcreteModelType::CLIP {
            let handler = Self::get_multi_modal_embedding(ctx)?;

            return Ok((&handler).into());
        }

        let handler = AIModel::new(
            move || {
                let resources_dir_clone = resources_dir.clone();
                let model_clone = model.clone();
                async move {
                    let params = model_clone.params;
                    match model_clone.model_type {
                        ConcreteModelType::OrtTextEmbedding => {
                            let model_path = resources_dir_clone
                                .join(get_str_from_params(&params, "model_path")?);
                            let tokenizer_config_path = resources_dir_clone
                                .join(get_str_from_params(&params, "tokenizer_config_path")?);
                            OrtTextEmbedding::new(model_path, tokenizer_config_path).await
                        }
                        _ => {
                            bail!(
                                "unsupported model {} for multi modal embedding",
                                model_clone.model_type.as_ref()
                            )
                        }
                    }
                }
            },
            Some(Duration::from_secs(30)),
        )
        .map_err(|e| rspc::Error::new(rspc::ErrorCode::InternalServerError, e.to_string()))?;

        Ok(handler)
    }

    fn get_llm(ctx: &dyn CtxWithLibrary) -> Result<LLMModel, rspc::Error> {
        let resources_dir = ctx.get_resources_dir().to_path_buf();
        let library = ctx.library()?;
        let settings = get_library_settings(&library.dir);

        let model = get_model_info_by_id(ctx, &settings.models.llm)?;
        let handler = AIModel::new(
            move || {
                let resources_dir_clone = resources_dir.clone();
                let model_clone = model.clone();

                async move {
                    let params = model_clone.params;
                    match model_clone.model_type {
                        ConcreteModelType::Qwen2 => {
                            let model_path = resources_dir_clone
                                .join(get_str_from_params(&params, "model_path")?);
                            let tokenizer_path = resources_dir_clone
                                .join(get_str_from_params(&params, "tokenizer_path")?);
                            let device = get_str_from_params(&params, "device")?;

                            Qwen2::load(&model_path, &tokenizer_path, &device)
                                .map(|v| LLM::Qwen2(v))
                        }
                        _ => {
                            bail!(
                                "unsupported model {} for LLM",
                                model_clone.model_type.as_ref()
                            )
                        }
                    }
                }
            },
            Some(Duration::from_secs(30)),
        )
        .map_err(|e| rspc::Error::new(rspc::ErrorCode::InternalServerError, e.to_string()))?;

        Ok(handler)
    }

    pub fn update_multi_modal_embedding(&mut self, ctx: &dyn CtxWithLibrary) {
        let _ = self.multi_modal_embedding.shutdown();
        self.multi_modal_embedding =
            Self::get_multi_modal_embedding(ctx).expect("failed to get multi modal embedding");
    }

    pub fn update_text_embedding(&mut self, ctx: &dyn CtxWithLibrary) {
        let _ = self.text_embedding.shutdown();
        self.text_embedding = Self::get_text_embedding(ctx, &self.multi_modal_embedding)
            .expect("failed to get text embedding");
    }

    pub fn update_image_caption(&mut self, ctx: &dyn CtxWithLibrary) {
        let _ = self.image_caption.shutdown();
        self.image_caption = Self::get_image_caption(ctx).expect("");
    }

    pub fn update_audio_transcript(&mut self, ctx: &dyn CtxWithLibrary) {
        let _ = self.audio_transcript.shutdown();
        self.audio_transcript =
            Self::get_audio_transcript(ctx).expect("failed to get audio transcript");
    }

    pub async fn shutdown(&self) -> anyhow::Result<()> {
        self.multi_modal_embedding.shutdown().await?;
        self.text_embedding.shutdown().await?;
        self.image_caption.shutdown().await?;
        self.audio_transcript.shutdown().await?;
        self.llm.shutdown().await?;

        Ok(())
    }
}
