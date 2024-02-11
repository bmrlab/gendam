use rspc::Router;
use crate::{Ctx, R};
use tracing::debug;

pub fn get_routes() -> Router<Ctx> {
    R.router()
        .procedure(
            "create_video_frames",
            R.mutation(|ctx, video_path: String| async move {
                let res = create_video_frames(&ctx, &video_path).await;
                serde_json::to_value(res).unwrap()
            })
        )
}

async fn create_video_frames(ctx: &Ctx, video_path: &str) {
    let video_handler =
        file_handler::video::VideoHandler::new(
            video_path,
            &ctx.local_data_dir,
            &ctx.resources_dir,
        )
        .await
        .expect("failed to initialize video handler");
    let frame_handle = tokio::spawn(async move {
        match video_handler.get_frames().await {
            Ok(res) => {
                debug!("successfully got frames");
                Ok(res)
            },
            Err(e) => {
                debug!("failed to get frames: {}", e);
                Err(e)
            }
        }
    });
    let result = frame_handle.await.unwrap();
    result.expect("failed to get frames");
}
