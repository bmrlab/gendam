use super::search::SearchResultPayload;
use content_base::ContentBase;
use content_library::Library;
use serde::Deserialize;
use specta::Type;

#[derive(Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct RecommendRequestPayload {
    pub asset_object_hash: String,
    pub timestamp: i32,
}

pub async fn recommend_frames(
    _library: &Library,
    _content_base: &ContentBase,
    _asset_object_hash: &str,
    _timestamp: i32,
) -> Result<Vec<SearchResultPayload>, rspc::Error> {
    // let payload = RecommendVideoFramePayload::new(asset_object_hash, timestamp as i64);
    // let search_results = content_base
    //     .recommend_video_frame(payload)
    //     .await
    //     .map_err(|e| {
    //         tracing::error!("Failed to recommend frames: {e}");
    //         rspc::Error::new(
    //             rspc::ErrorCode::InternalServerError,
    //             format!("Failed to recommend frames: {e}"),
    //         )
    //     })?;

    // let result = retrieve_assets_for_search(library, search_results).await?;
    // Ok(result)
    todo!()
}
