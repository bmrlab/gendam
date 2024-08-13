pub mod task;

use crate::CtxWithLibrary;
use content_handler::video::VideoDecoder;
use prisma_lib::asset_object;
use rspc::{Router, RouterBuilder};
use serde::{Deserialize, Serialize};
use specta::Type;

pub fn get_routes<TCtx>() -> RouterBuilder<TCtx>
where
    TCtx: CtxWithLibrary + Clone + Send + Sync + 'static,
{
    Router::new()
        .merge("tasks.", task::get_routes::<TCtx>())
        .query("player.video_info", |t| {
            #[derive(Deserialize, Type, Debug)]
            #[serde(rename_all = "camelCase")]
            struct VideoPlayerInfoRequestPayload {
                hash: String,
            }
            #[derive(Serialize, Type, Debug)]
            #[serde(rename_all = "camelCase")]
            struct VideoPlayerInfoResponse {
                hash: String,
                duration: f64,
                mime_type: Option<String>,
                has_video: bool,
                has_audio: bool,
            }
            t(
                |ctx: TCtx, input: VideoPlayerInfoRequestPayload| async move {
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
                            rspc::Error::new(
                                rspc::ErrorCode::InternalServerError,
                                format!("{}", err),
                            )
                        })?;

                    if let None = asset_object_data {
                        return Err(rspc::Error::new(
                            rspc::ErrorCode::InternalServerError,
                            format!("asset no found"),
                        ));
                    };

                    let asset_object_data = asset_object_data.unwrap();

                    let video_path = library.file_path(&asset_object_data.hash);

                    let video_decoder = VideoDecoder::new(&video_path).map_err(|e| {
                        rspc::Error::new(
                            rspc::ErrorCode::InternalServerError,
                            format!("failed to get video metadata: {}", e),
                        )
                    })?;

                    let (has_video, has_audio) =
                        video_decoder.check_video_audio().await.map_err(|e| {
                            rspc::Error::new(
                                rspc::ErrorCode::InternalServerError,
                                format!("failed to check video: {}", e),
                            )
                        })?;

                    match video_decoder.get_video_duration().await {
                        Ok(duration) => Ok(VideoPlayerInfoResponse {
                            hash: input.hash,
                            duration,
                            mime_type: asset_object_data.mime_type,
                            has_video,
                            has_audio,
                        }),
                        Err(e) => Err(rspc::Error::new(
                            rspc::ErrorCode::InternalServerError,
                            format!("failed to get video metadata: {}", e),
                        )),
                    }
                },
            )
        })
        .query("player.video_ts", |t| {
            #[derive(Deserialize, Type, Debug)]
            #[serde(rename_all = "camelCase")]
            struct VideoPlayerTsRequestPayload {
                hash: String,
                index: u32,
            }
            #[derive(Serialize, Type, Debug)]
            #[serde(rename_all = "camelCase")]
            struct VideoPlayerTsResponse {
                data: Vec<u8>,
            }
            t(|ctx: TCtx, input: VideoPlayerTsRequestPayload| async move {
                let library = ctx.library()?;
                // let temp_dir = ctx.get_temp_dir();
                let cache_dir = ctx.get_cache_dir();
                let ts_dir = cache_dir;

                let video_path = library.file_path(&input.hash);
                let video_decoder = VideoDecoder::new(video_path).map_err(|e| {
                    rspc::Error::new(
                        rspc::ErrorCode::InternalServerError,
                        format!("failed to get video metadata: {}", e),
                    )
                })?;

                let file = video_decoder
                    .generate_ts(input.index, ts_dir)
                    .await
                    .map_err(|e| {
                        rspc::Error::new(
                            rspc::ErrorCode::InternalServerError,
                            format!("failed to get ts file: {}", e),
                        )
                    })?;

                Ok(VideoPlayerTsResponse { data: file })
            })
        })
}
