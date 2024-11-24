pub(crate) mod models;
use self::models::{get_model_info_by_id, ConcreteModelType};
use crate::{ctx::traits::CtxWithLibrary, library::get_library_settings};
use ai::{
    blip::BLIP,
    clip::{CLIPModel, CLIP},
    llava_phi3_mini::LLaVAPhi3Mini,
    llm::{openai::OpenAI, qllama::Qllama, qwen2::Qwen2, LLM},
    text_embedding::OrtTextEmbedding,
    whisper::Whisper,
    AIModel, AudioTranscriptModel, ImageCaptionModel, LLMModel, MultiModalEmbeddingModel,
    TextEmbeddingModel,
};
use serde_json::Value;
use std::{fmt, time::Duration};

/// AIHandler manages different AI models used in the application.
///
/// The default model selections for each category are defined in the `impl Default for LibraryModels`,
/// located in `apps/api-server/src/library.rs`.
///
/// These defaults can be overridden by user settings in the library settings file,
/// found at `<library_dir>/settings.json`.
///
/// Available models and their configurations are defined in the `model_list.json` file
/// located in the resources directory.
///
/// The actual model selection and instantiation occurs in the corresponding `build_*_model` methods
/// (e.g., `build_image_caption_model`, `build_multi_modal_embedding_model`, etc.) based on the following priority:
/// 1. User-specified settings in the library settings file
/// 2. Default values from `impl Default for LibraryModels`
/// 3. Model configurations from `model_list.json`
#[derive(Clone)]
pub struct AIHandler {
    pub multi_modal_embedding: (MultiModalEmbeddingModel, String),
    pub image_caption: (ImageCaptionModel, String),
    pub audio_transcript: (AudioTranscriptModel, String),
    pub text_embedding: (TextEmbeddingModel, String),
    pub llm: (LLMModel, String),
    /// 目前这个是专门给 audio transcript 和 raw text 的 chunking 用的
    pub text_tokenizer: (ai::tokenizers::Tokenizer, String),
}

impl fmt::Debug for AIHandler {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("AIHandler").finish()
    }
}

fn get_str_from_params<'a>(params: &'a Value, name: &str) -> anyhow::Result<&'a str> {
    match params[name].as_str() {
        Some(s) => Ok(s),
        _ => Err(anyhow::anyhow!("Invalid model config {}", name)),
    }
}

impl AIHandler {
    pub fn new(ctx: &dyn CtxWithLibrary) -> anyhow::Result<Self> {
        let multi_modal_embedding = Self::build_multi_modal_embedding_model(ctx)?;
        let text_embedding = Self::build_text_embedding_model(
            ctx,
            (&multi_modal_embedding.0, &multi_modal_embedding.1),
        )?;
        let llm = Self::build_llm_model(ctx)?;
        let text_tokenizer = Self::build_text_tokenizer(ctx)?;
        let image_caption = Self::build_image_caption_model(ctx)?;
        let audio_transcript = Self::build_audio_transcript_model(ctx)?;

        Ok(Self {
            multi_modal_embedding,
            image_caption,
            audio_transcript,
            text_embedding,
            llm,
            text_tokenizer,
        })
    }

    fn build_image_caption_model(
        ctx: &dyn CtxWithLibrary,
    ) -> anyhow::Result<(ImageCaptionModel, String)> {
        let resources_dir = ctx.get_resources_dir().to_path_buf();
        let library = ctx.library()?;
        let settings = get_library_settings(&library.dir);

        let model = get_model_info_by_id(ctx, &settings.models.image_caption)?;
        let model_id = model.id.clone();

        let handler = if model.model_type == ConcreteModelType::OpenAI {
            let handler = AIModel::new(
                model_id.clone(),
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
            )?;

            // 这里是使用 LLM 进行 image caption 的系统提示词
            // const LLM_IMAGE_CAPTION_SYSTEM_PROMPT: &'static str = r#"Describe People (including famous individuals), Actions, Objects, Animals or pets, Nature, Sounds (excluding human speech) in the image."#;
            // handler.create_image_caption_ref(LLM_IMAGE_CAPTION_SYSTEM_PROMPT)
            handler.create_image_caption_ref()
        } else {
            // Model trait 的 process 返回的是 impl Future, 导致 Model trait 不是 object safe 的
            // 这里没法用 Box<dyn Model<Item = ImageCaptionInput, Output = ImageCaptionOutput>> 来接收每个模型的实例
            // 所以只能把 match 写在外面，把模型实例直接传给 AIModel::new
            let handler = match model.model_type {
                ConcreteModelType::BLIP => AIModel::new(
                    model_id.clone(),
                    move || {
                        let resources_dir_clone = resources_dir.clone();
                        let model_clone = model.clone();
                        async move {
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
                    },
                    Some(Duration::from_secs(30)),
                )?,
                ConcreteModelType::LLaVAPhi3Mini => AIModel::new(
                    model_id.clone(),
                    move || {
                        let resources_dir_clone = resources_dir.clone();
                        let model_clone = model.clone();
                        async move {
                            // TODO 类型这里可以扩展下，支持 LLaVA 的模型而不只是 LLaVAPhi3Mini，具体实现可以分模型
                            let params = model_clone.params;
                            let model_path = resources_dir_clone
                                .join(get_str_from_params(&params, "model_path")?);
                            let mmproj_model_path = resources_dir_clone
                                .join(get_str_from_params(&params, "mmproj_model_path")?);
                            let tokenizer_path = resources_dir_clone
                                .join(get_str_from_params(&params, "tokenizer_path")?);
                            let preprocessor_config_path = resources_dir_clone
                                .join(get_str_from_params(&params, "preprocessor_config_path")?);
                            let device = get_str_from_params(&params, "device")?;
                            LLaVAPhi3Mini::new(
                                device,
                                model_path,
                                mmproj_model_path,
                                tokenizer_path,
                                preprocessor_config_path,
                            )
                        }
                    },
                    Some(Duration::from_secs(30)),
                )?,
                _ => anyhow::bail!(
                    "unsupported model {} for image caption",
                    model.model_type.as_ref(),
                ),
            };

            handler
        };

        Ok((handler, model_id))
    }

    fn build_multi_modal_embedding_model(
        ctx: &dyn CtxWithLibrary,
    ) -> anyhow::Result<(MultiModalEmbeddingModel, String)> {
        let resources_dir = ctx.get_resources_dir().to_path_buf();
        let library = ctx.library()?;
        let settings = get_library_settings(&library.dir);

        let model = get_model_info_by_id(ctx, &settings.models.multi_modal_embedding)?;
        let model_id = model.id.clone();

        let handler = AIModel::new(
            model_id.clone(),
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
                            anyhow::bail!(
                                "unsupported model {} for multi modal embedding",
                                model_clone.model_type.as_ref()
                            )
                        }
                    }
                }
            },
            Some(Duration::from_secs(600)),
        )?;

        Ok((handler, model_id))
    }

    fn build_audio_transcript_model(
        ctx: &dyn CtxWithLibrary,
    ) -> anyhow::Result<(AudioTranscriptModel, String)> {
        let resources_dir = ctx.get_resources_dir().to_path_buf();
        let library = ctx.library()?;
        let settings = get_library_settings(&library.dir);

        let model = get_model_info_by_id(ctx, &settings.models.audio_transcript)?;
        let model_id = model.id.clone();

        let handler = AIModel::new(
            model_id.clone(),
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
                            anyhow::bail!(
                                "unsupported model {} for multi modal embedding",
                                model_clone.model_type.as_ref()
                            )
                        }
                    }
                }
            },
            Some(Duration::from_secs(30)),
        )?;

        Ok((handler, model_id))
    }

    /// Get text embedding model.
    ///
    /// ⚠️ 因为 multi_modal_embedding_model 也能完成 text_embedding，所以这里也传入他，避免重复加载同样的模型
    fn build_text_embedding_model(
        ctx: &dyn CtxWithLibrary,
        multi_modal_handler: (&MultiModalEmbeddingModel, &str),
    ) -> anyhow::Result<(TextEmbeddingModel, String)> {
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
        let model_id = model.id.clone();

        if model.model_type == ConcreteModelType::CLIP {
            let (handler, name) = Self::build_multi_modal_embedding_model(ctx)?;
            return Ok(((&handler).into(), name));
        }

        let handler = AIModel::new(
            model_id.clone(),
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
                            anyhow::bail!(
                                "unsupported model {} for multi modal embedding",
                                model_clone.model_type.as_ref()
                            )
                        }
                    }
                }
            },
            Some(Duration::from_secs(600)),
        )?;

        Ok((handler, model_id))
    }

    /// 目前这个是专门给 audio transcript 和 raw text 的 chunking 用的
    fn build_text_tokenizer(
        ctx: &dyn CtxWithLibrary,
    ) -> anyhow::Result<(ai::tokenizers::Tokenizer, String)> {
        let resources_dir = ctx.get_resources_dir().to_path_buf();
        let library = ctx.library()?;
        let settings = get_library_settings(&library.dir);

        let model = get_model_info_by_id(ctx, &settings.models.llm)?;

        let (tokenizer_path, name) = match model.model_type {
            ConcreteModelType::Qwen2 => (
                get_str_from_params(&model.params, "tokenizer_path")?,
                model.id.as_str(),
            ),
            ConcreteModelType::OpenAI | ConcreteModelType::AzureOpenAI => {
                // TODO: LLM Service 不需要 tokenizer，但是 audio transcript 和 raw text 的 chunking 需要，这里设置个默认的，回头优化
                ("./qwen2/tokenizer.json", "default")
            }
            _ => ("./qwen2/tokenizer.json", "default"),
        };
        let tokenizer = ai::tokenizers::Tokenizer::from_file(resources_dir.join(tokenizer_path))
            .map_err(|e| {
                anyhow::anyhow!("Failed to load tokenizer from {}: {}", tokenizer_path, e)
            })?;
        Ok((tokenizer, name.to_string()))
    }

    fn build_llm_model(ctx: &dyn CtxWithLibrary) -> anyhow::Result<(LLMModel, String)> {
        let resources_dir = ctx.get_resources_dir().to_path_buf();
        let library = ctx.library()?;
        let settings = get_library_settings(&library.dir);

        let model = get_model_info_by_id(ctx, &settings.models.llm)?;
        let model_id = model.id.clone();

        let handler = AIModel::new(
            model_id.clone(),
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
                        ConcreteModelType::LLaVAPhi3Mini => {
                            // 和 LLaVAPhi3 使用同一个模型，但是不需要 mmproj
                            let model_path = resources_dir_clone
                                .join(get_str_from_params(&params, "model_path")?);
                            let tokenizer_path = resources_dir_clone
                                .join(get_str_from_params(&params, "tokenizer_path")?);
                            let device = get_str_from_params(&params, "device")?;
                            Qllama::load(model_path, tokenizer_path, device).map(|v| LLM::Qllama(v))
                        }
                        ConcreteModelType::OpenAI => {
                            let base_url = get_str_from_params(&params, "base_url")?;
                            let api_key = get_str_from_params(&params, "api_key")?;
                            let model_name = get_str_from_params(&params, "model")?;

                            OpenAI::new(base_url, api_key, model_name).map(|v| LLM::OpenAI(v))
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
                            anyhow::bail!(
                                "unsupported model {} for LLM",
                                model_clone.model_type.as_ref()
                            )
                        }
                    }
                }
            },
            Some(Duration::from_secs(30)),
        )?;

        Ok((handler, model_id))
    }

    pub fn rebuild_multi_modal_embedding_model(
        &mut self,
        ctx: &dyn CtxWithLibrary,
    ) -> anyhow::Result<()> {
        self.multi_modal_embedding = Self::build_multi_modal_embedding_model(ctx)?;
        self.rebuild_text_embedding_model(ctx)?;
        Ok(())
    }

    pub fn rebuild_text_embedding_model(&mut self, ctx: &dyn CtxWithLibrary) -> anyhow::Result<()> {
        self.text_embedding = Self::build_text_embedding_model(
            ctx,
            (&self.multi_modal_embedding.0, &self.multi_modal_embedding.1),
        )?;
        Ok(())
    }

    pub fn rebuild_llm_model(&mut self, ctx: &dyn CtxWithLibrary) -> anyhow::Result<()> {
        self.llm = Self::build_llm_model(ctx)?;
        Ok(())
    }

    pub fn rebuild_image_caption_model(&mut self, ctx: &dyn CtxWithLibrary) -> anyhow::Result<()> {
        self.image_caption = Self::build_image_caption_model(ctx)?;
        Ok(())
    }

    pub fn rebuild_audio_transcript_model(
        &mut self,
        ctx: &dyn CtxWithLibrary,
    ) -> anyhow::Result<()> {
        self.audio_transcript = Self::build_audio_transcript_model(ctx)?;
        Ok(())
    }
}
