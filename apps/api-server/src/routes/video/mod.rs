use crate::CtxWithLibrary;
use content_handler::video::VideoDecoder;
use rspc::{Router, RouterBuilder};
use serde::{Deserialize, Serialize};
use specta::Type;

pub fn get_routes<TCtx>() -> RouterBuilder<TCtx>
where
    TCtx: CtxWithLibrary + Clone + Send + Sync + 'static,
{
    Router::new().query("player.video_ts", |t| {
        #[derive(Deserialize, Type, Debug)]
        #[serde(rename_all = "camelCase")]
        struct VideoPlayerTsRequestPayload {
            hash: String,
            index: u32,
            size: u32,
        }
        #[derive(Serialize, Type, Debug)]
        #[serde(rename_all = "camelCase")]
        struct VideoPlayerTsResponse {
            data: Vec<u8>,
        }
        t(|ctx: TCtx, input: VideoPlayerTsRequestPayload| async move {
            let library = ctx.library()?;

            let video_path = library.file_path(&input.hash);
            let video_decoder = VideoDecoder::new(video_path).map_err(|e| {
                rspc::Error::new(
                    rspc::ErrorCode::InternalServerError,
                    format!("failed to get video metadata: {}", e),
                )
            })?;

            let file = video_decoder
                .generate_ts(input.index, input.size)
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
