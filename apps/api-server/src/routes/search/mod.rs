use crate::CtxWithLibrary;
use glob::glob;
use rspc::{Router, RouterBuilder};

mod recommend;
mod search;
use recommend::{recommend_frames, RecommendRequestPayload};
use search::{search_all, SearchRequestPayload};

pub fn get_routes<TCtx>() -> RouterBuilder<TCtx>
where
    TCtx: CtxWithLibrary + Clone + Send + Sync + 'static,
{
    Router::<TCtx>::new()
        .query("all", |t| {
            t(|ctx: TCtx, input: SearchRequestPayload| async move {
                let library = ctx.library()?;
                let qdrant_info = ctx.qdrant_info()?;
                let ai_handler = ctx.ai_handler()?;
                search_all(&library, &qdrant_info, &ai_handler, input).await
            })
        })
        .query("recommend", |t| {
            t(|ctx: TCtx, input: RecommendRequestPayload| async move {
                let library = ctx.library()?;
                let qdrant_info = ctx.qdrant_info()?;
                recommend_frames(
                    &library,
                    &qdrant_info,
                    &input.asset_object_hash,
                    input.timestamp,
                )
                .await
            })
        })
        .query("suggestions", |t| {
            t(|ctx: TCtx, _input: ()| async move {
                let library = ctx.library()?;
                // let asset_object_data_list = library
                //     .prisma_client()
                //     .asset_object()
                //     .find_many(vec![])
                //     .exec()
                //     .await
                //     .map_err(sql_error)?;
                // let captions = asset_object_data_list
                //     .into_iter()
                //     .filter_map(|asset_object_data| {
                //         // let video_handler = VideoHandler::new(
                //         //     &asset_object_data.hash, &library
                //         // ).ok()?;
                //         // video_handler.get_artifacts_settings().ok()?;
                //         Some("".to_string())
                //     })
                //     .collect::<Vec<String>>();
                let pattern = format!(
                    "{}/artifacts/*/*/frame-caption-*/*.json",
                    library.dir.to_string_lossy()
                );
                let entries = glob(&pattern).map_err(|e| {
                    rspc::Error::new(
                        rspc::ErrorCode::InternalServerError,
                        format!("glob failed: {}", e),
                    )
                })?;
                let results = entries
                    .into_iter()
                    .filter_map(|entry| {
                        let json_path = entry.ok()?;
                        let json_str = std::fs::read_to_string(&json_path).ok()?;
                        let json_val = serde_json::from_str::<serde_json::Value>(&json_str).ok()?;
                        let caption = json_val.get("caption")?.as_str()?;
                        Some(caption.to_owned())
                    })
                    .collect::<Vec<String>>();
                Ok(results)
            })
        })
}
