pub mod task;

use crate::CtxWithLibrary;
use file_handler::video::VideoHandler;
use prisma_lib::asset_object;
use rspc::{Router, RouterBuilder};
use serde::Deserialize;
use serde_json::json;
use specta::Type;

pub fn get_routes<TCtx>() -> RouterBuilder<TCtx>
where
    TCtx: CtxWithLibrary + Clone + Send + Sync + 'static,
{
    Router::new()
        .merge("tasks.", task::get_routes::<TCtx>())
        .mutation("get_video_info", |t| {
            #[derive(Deserialize, Type, Debug)]
            #[serde(rename_all = "camelCase")]
            struct VideoRequestPayload {
                hash: String,
            }

            t(|ctx: TCtx, input: VideoRequestPayload| async move {
                let library = ctx.library()?;
                let asset_object_data = library
                    .prisma_client()
                    .asset_object()
                    .find_unique(asset_object::UniqueWhereParam::HashEquals(
                        input.hash.clone(),
                    ))
                    .exec()
                    .await
                    .map_err(|err| {
                        rspc::Error::new(rspc::ErrorCode::InternalServerError, format!("{}", err))
                    })?;

                if let None = asset_object_data {
                    return Err(rspc::Error::new(
                        rspc::ErrorCode::InternalServerError,
                        format!("asset no found"),
                    ));
                };

                let asset_object_data = asset_object_data.unwrap();

                let video_handler = VideoHandler::new(&input.hash, &library).map_err(|e| {
                    rspc::Error::new(
                        rspc::ErrorCode::InternalServerError,
                        format!("failed to get video metadata: {}", e),
                    )
                })?;

                // let _ = video_handler.get_hls().await;

                match video_handler.get_video_duration().await {
                    Ok(duration) => Ok(json!({
                        "hash": input.hash,
                        "duration": duration,
                        "mimeType": asset_object_data.mime_type
                    })),
                    Err(e) => Err(rspc::Error::new(
                        rspc::ErrorCode::InternalServerError,
                        format!("failed to get video metadata: {}", e),
                    )),
                }
            })
        })
        .mutation("get_ts", |t| {
            #[derive(Deserialize, Type, Debug)]
            #[serde(rename_all = "camelCase")]
            struct TsRequestPayload {
                hash: String,
                index: u32,
            }
            t(|ctx: TCtx, input: TsRequestPayload| async move {
                let library = ctx.library()?;

                let video_handler =
                    VideoHandler::new(&input.hash.clone(), &library).map_err(|e| {
                        rspc::Error::new(
                            rspc::ErrorCode::InternalServerError,
                            format!("failed to get video metadata: {}", e),
                        )
                    })?;

                let file = video_handler.generate_ts(input.index).await.map_err(|e| {
                    rspc::Error::new(
                        rspc::ErrorCode::InternalServerError,
                        format!("failed to get ts file: {}", e),
                    )
                })?;

                Ok(json!({
                    "data": file
                }))
            })
        })
}
