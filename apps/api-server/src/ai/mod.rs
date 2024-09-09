pub mod models;

use self::models::{get_model_info_by_id, ConcreteModelType};
use crate::{library::get_library_settings, CtxWithLibrary};
use ai::{
    blip::BLIP,
    clip::{CLIPModel, CLIP},
    llm::{openai::OpenAI, qwen2::Qwen2, LLM},
    text_embedding::OrtTextEmbedding,
    whisper::Whisper,
    AIModel, AudioTranscriptModel, ImageCaptionModel, LLMModel, MultiModalEmbeddingModel,
    TextEmbeddingModel,
};
use anyhow::bail;
use serde_json::Value;
use std::{fmt, time::Duration};

#[derive(Clone)]
pub struct AIHandler {
    pub multi_modal_embedding: (MultiModalEmbeddingModel, String),
    pub image_caption: (ImageCaptionModel, String),
    pub audio_transcript: (AudioTranscriptModel, String),
    pub text_embedding: (TextEmbeddingModel, String),
    pub llm: (LLMModel, ai::tokenizers::Tokenizer, String),
}

impl fmt::Debug for AIHandler {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("AIHandler").finish()
    }
}

fn get_str_from_params<'a>(params: &'a Value, name: &str) -> Result<&'a str, rspc::Error> {
    match params[name].as_str() {
        Some(s) => Ok(s),
        _ => Err(rspc::Error::new(
            rspc::ErrorCode::InternalServerError,
            format!("invalid {}", name),
        )),
    }
}

impl AIHandler {
    pub fn new(ctx: &dyn CtxWithLibrary) -> Result<Self, rspc::Error> {
        let multi_modal_embedding = Self::get_multi_modal_embedding(ctx)?;
        let text_embedding =
            Self::get_text_embedding(ctx, (&multi_modal_embedding.0, &multi_modal_embedding.1))?;
        let llm = Self::get_llm(ctx)?;

        Ok(Self {
            multi_modal_embedding,
            image_caption: Self::get_image_caption(ctx)?,
            audio_transcript: Self::get_audio_transcript(ctx)?,
            text_embedding,
            llm,
        })
    }

    fn get_image_caption(
        ctx: &dyn CtxWithLibrary,
    ) -> Result<(ImageCaptionModel, String), rspc::Error> {
        let resources_dir = ctx.get_resources_dir().to_path_buf();
        let library = ctx.library()?;
        let settings = get_library_settings(&library.dir);

        let model = get_model_info_by_id(ctx, &settings.models.image_caption)?;
        let model_name = model.id.clone();

        let handler = if model.model_type == ConcreteModelType::OpenAI {
            let handler = AIModel::new(
                move || {
                    let model_clone = model.clone();

                    async move {
                        let params = model_clone.params;
                        let base_url = get_str_from_params(&params, "base_url")?;
                        let api_key = get_str_from_params(&params, "api_key")?;
                        let model = get_str_from_params(&params, "model")?;

                        OpenAI::new(base_url, api_key, model).map(|v| LLM::OpenAI(v))
                    }
                },
                Some(Duration::from_secs(30)),
            )
            .map_err(|e| rspc::Error::new(rspc::ErrorCode::InternalServerError, e.to_string()))?;

            handler.create_image_caption_ref("Please describe the image.")
        } else {
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
                                let model_type = match model_type {
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

            handler
        };

        Ok((handler, model_name))
    }

    fn get_multi_modal_embedding(
        ctx: &dyn CtxWithLibrary,
    ) -> Result<(MultiModalEmbeddingModel, String), rspc::Error> {
        let resources_dir = ctx.get_resources_dir().to_path_buf();
        let library = ctx.library()?;
        let settings = get_library_settings(&library.dir);

        let model = get_model_info_by_id(ctx, &settings.models.multi_modal_embedding)?;
        let model_name = model.id.clone();
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
            Some(Duration::from_secs(600)),
        )
        .map_err(|e| rspc::Error::new(rspc::ErrorCode::InternalServerError, e.to_string()))?;

        Ok((handler, model_name))
    }

    fn get_audio_transcript(
        ctx: &dyn CtxWithLibrary,
    ) -> Result<(AudioTranscriptModel, String), rspc::Error> {
        let resources_dir = ctx.get_resources_dir().to_path_buf();
        let library = ctx.library()?;
        let settings = get_library_settings(&library.dir);

        let model = get_model_info_by_id(ctx, &settings.models.audio_transcript)?;
        let model_name = model.id.clone();
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

        Ok((handler, model_name))
    }

    /// Get text embedding model.
    ///
    /// ⚠️ 因为 multi_modal_embedding_model 也能完成 text_embedding，所以这里也传入他，避免重复加载同样的模型
    fn get_text_embedding(
        ctx: &dyn CtxWithLibrary,
        multi_modal_handler: (&MultiModalEmbeddingModel, &str),
    ) -> Result<(TextEmbeddingModel, String), rspc::Error> {
        let resources_dir = ctx.get_resources_dir().to_path_buf();
        let library = ctx.library()?;
        let settings = get_library_settings(&library.dir);

        if settings.models.text_embedding == settings.models.multi_modal_embedding {
            return Ok((
                multi_modal_handler.0.into(),
                multi_modal_handler.1.to_string(),
            ));
        }

        let model = get_model_info_by_id(ctx, &settings.models.text_embedding)?;
        let model_name = model.id.clone();

        if model.model_type == ConcreteModelType::CLIP {
            let (handler, name) = Self::get_multi_modal_embedding(ctx)?;
            return Ok(((&handler).into(), name));
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
            Some(Duration::from_secs(600)),
        )
        .map_err(|e| rspc::Error::new(rspc::ErrorCode::InternalServerError, e.to_string()))?;

        Ok((handler, model_name))
    }

    fn get_llm(
        ctx: &dyn CtxWithLibrary,
    ) -> Result<(LLMModel, ai::tokenizers::Tokenizer, String), rspc::Error> {
        let resources_dir = ctx.get_resources_dir().to_path_buf();
        let library = ctx.library()?;
        let settings = get_library_settings(&library.dir);

        let model = get_model_info_by_id(ctx, &settings.models.llm)?;
        let model_name = model.id.clone();
        let tokenizer_path = get_str_from_params(&model.params, "tokenizer_path")?;
        let tokenizer_path = resources_dir.join(tokenizer_path);

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
                        ConcreteModelType::OpenAI => {
                            let base_url = get_str_from_params(&params, "base_url")?;
                            let api_key = get_str_from_params(&params, "api_key")?;
                            let model = get_str_from_params(&params, "model")?;

                            OpenAI::new(base_url, api_key, model).map(|v| LLM::OpenAI(v))
                        }
                        ConcreteModelType::AzureOpenAI => {
                            let azure_endpoint = get_str_from_params(&params, "azure_endpoint")?;
                            let api_key = get_str_from_params(&params, "api_key")?;
                            let deployment_name = get_str_from_params(&params, "deployment_name")?;
                            let api_version = get_str_from_params(&params, "api_version")?;

                            OpenAI::new_azure(azure_endpoint, api_key, deployment_name, api_version)
                                .map(|v| LLM::OpenAI(v))
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

        // TODO here use a fake tokenizer for now, should be updated in the future
        let tokenizer = ai::tokenizers::Tokenizer::from_file(tokenizer_path)
            .map_err(|e| rspc::Error::new(rspc::ErrorCode::InternalServerError, e.to_string()))?;

        Ok((handler, tokenizer, model_name))
    }

    pub fn update_multi_modal_embedding(&mut self, ctx: &dyn CtxWithLibrary) {
        self.multi_modal_embedding =
            Self::get_multi_modal_embedding(ctx).expect("failed to get multi modal embedding");
        self.update_text_embedding(ctx);
    }

    pub fn update_text_embedding(&mut self, ctx: &dyn CtxWithLibrary) {
        self.text_embedding = Self::get_text_embedding(
            ctx,
            (&self.multi_modal_embedding.0, &self.multi_modal_embedding.1),
        )
        .expect("failed to get text embedding");
    }

    pub fn update_image_caption(&mut self, ctx: &dyn CtxWithLibrary) {
        self.image_caption = Self::get_image_caption(ctx).expect("");
    }

    pub fn update_audio_transcript(&mut self, ctx: &dyn CtxWithLibrary) {
        self.audio_transcript =
            Self::get_audio_transcript(ctx).expect("failed to get audio transcript");
    }
}
