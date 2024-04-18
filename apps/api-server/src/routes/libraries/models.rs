use crate::{
    ai::models::{
        get_model_info_by_id, get_model_status, load_model_list, trigger_model_download,
        AIModelCategory, AIModelResult,
    },
    library::{get_library_settings, set_library_settings},
    CtxWithLibrary,
};
use content_library::make_sure_collection_created;
use rspc::{Router, RouterBuilder};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::thread::sleep;
use vector_db::get_language_collection_name;

pub fn get_routes<TCtx>() -> RouterBuilder<TCtx>
where
    TCtx: CtxWithLibrary + Clone + Send + Sync + 'static,
{
    Router::<TCtx>::new()
        .query("list", |t| {
            t(|ctx, _: ()| {
                let resources_dir = ctx.get_resources_dir();
                let model_list = load_model_list(&resources_dir)?;

                // 按类别分组
                let mut models_by_category: std::collections::HashMap<
                    AIModelCategory,
                    Vec<AIModelResult>,
                > = std::collections::HashMap::new();
                for model in model_list {
                    let categories = model.categories.clone();
                    for category in categories {
                        models_by_category
                            .entry(category)
                            .or_insert_with(Vec::new)
                            .push(AIModelResult {
                                info: model.clone(),
                                status: get_model_status(&ctx, &model),
                            });
                    }
                }

                #[derive(Serialize, Type)]
                #[serde(rename_all = "camelCase")]
                struct Result {
                    category: AIModelCategory,
                    models: Vec<AIModelResult>,
                }

                Ok(models_by_category
                    .into_iter()
                    .map(|(category, models)| Result {
                        category: category.clone(),
                        models,
                    })
                    .collect::<Vec<_>>())
            })
        })
        .mutation("set_model", |t| {
            #[derive(Deserialize, Type, Debug)]
            #[serde(rename_all = "camelCase")]
            struct SetModelPayload {
                category: AIModelCategory,
                model_id: String,
            }
            t(|ctx, payload: SetModelPayload| async move {
                let library = ctx.library()?;
                let mut settings = get_library_settings(&library.dir);

                match payload.category {
                    AIModelCategory::ImageCaption => {
                        settings.models.image_caption = payload.model_id;
                    }
                    AIModelCategory::AudioTranscript => {
                        settings.models.audio_transcript = payload.model_id;
                    }
                    AIModelCategory::TextEmbedding => {
                        let model_info = get_model_info_by_id(&ctx, &payload.model_id)?;
                        let dim = model_info.dim.ok_or(rspc::Error::new(
                            rspc::ErrorCode::InternalServerError,
                            "invalid model info".into(),
                        ))?;

                        if let Err(e) = make_sure_collection_created(
                            library.qdrant_client(),
                            &get_language_collection_name(&model_info.id),
                            dim as u64,
                        )
                        .await
                        {
                            return Err(rspc::Error::new(
                                rspc::ErrorCode::InternalServerError,
                                format!("failed to create qdrant collection: {}", e),
                            ));
                        }

                        settings.models.text_embedding = payload.model_id;
                    }
                    AIModelCategory::MultiModalEmbedding => {
                        settings.models.multi_modal_embedding = payload.model_id;
                    }
                    _ => {}
                }

                set_library_settings(&library.dir, settings);

                // manually trigger model update in ai_handler
                // this require to manipulate raw Mutex ai_handler
                let ai_handler = ctx.ai_handler_mutex();
                let mut ai_handler = ai_handler.lock().unwrap();
                if let Some(ai_handler) = &mut *ai_handler {
                    match payload.category {
                        AIModelCategory::TextEmbedding => {
                            ai_handler.update_text_embedding(&ctx);
                        }
                        AIModelCategory::MultiModalEmbedding => {
                            ai_handler.update_multi_modal_embedding(&ctx);
                        }
                        AIModelCategory::ImageCaption => {
                            ai_handler.update_image_caption(&ctx);
                        }
                        AIModelCategory::AudioTranscript => {
                            ai_handler.update_audio_transcript(&ctx);
                        }
                        _ => {}
                    }
                }

                Ok(())
            })
        })
        .mutation("download_model", |t| {
            #[derive(Deserialize, Type, Debug)]
            #[serde(rename_all = "camelCase")]
            struct DownloadModelPayload {
                model_id: String,
            }
            t(|ctx, payload: DownloadModelPayload| async move {
                let resources_dir = ctx.get_resources_dir();

                let model_list = load_model_list(&resources_dir)?;

                let model = model_list
                    .into_iter()
                    .find(|m| m.id == payload.model_id)
                    .ok_or_else(|| {
                        rspc::Error::new(
                            rspc::ErrorCode::NotFound,
                            format!("model not found: {}", payload.model_id),
                        )
                    })?;

                trigger_model_download(&resources_dir, &model, ctx.download_reporter()?)?;

                // 确保信息可以查到了再返回
                // TODO 这里有点影响性能，最简单的办法其实是在前端做个延迟
                let mut query_num = 0;
                loop {
                    let model_status = get_model_status(&ctx, &model);
                    if model_status.download_status.is_some() {
                        break;
                    }

                    query_num += 1;
                    if query_num > 3 {
                        return Err(rspc::Error::new(
                            rspc::ErrorCode::InternalServerError,
                            format!(
                                "model download not started successfully: {}",
                                payload.model_id
                            ),
                        ));
                    }

                    sleep(std::time::Duration::from_millis(200));
                }

                Ok(())
            })
        })
        .query("get_model", |t| {
            t(|ctx, model_id: String| async move {
                let resources_dir = ctx.get_resources_dir();
                let model_list = load_model_list(&resources_dir)?;

                let model_info = model_list
                    .iter()
                    .find(|m| m.id == model_id)
                    .ok_or_else(|| {
                        rspc::Error::new(
                            rspc::ErrorCode::NotFound,
                            format!("model not found: {}", model_id),
                        )
                    })?
                    .to_owned();

                let model_status = get_model_status(&ctx, &model_info);

                Ok(AIModelResult {
                    info: model_info,
                    status: model_status,
                })
            })
        })
}
