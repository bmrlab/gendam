use super::search::{retrieve_assets_for_search, SearchResultPayload};
use content_base::{query::RecommendPayload, ContentBase};
use content_library::{Library, QdrantServerInfo};
use serde::Deserialize;
use specta::Type;

#[derive(Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct RecommendRequestPayload {
    pub asset_object_hash: String,
    pub timestamp: i32,
}

pub async fn recommend_frames(
    library: &Library,
    content_base: &ContentBase,
    asset_object_hash: &str,
    timestamp: i32,
) -> Result<Vec<SearchResultPayload>, rspc::Error> {
    // TODO timestamp should be included in recommend payload
    let payload = RecommendPayload::new(asset_object_hash);
    let search_results = content_base.recommend(payload).await.map_err(|e| {
        tracing::error!("Failed to recommend frames: {e}");
        rspc::Error::new(
            rspc::ErrorCode::InternalServerError,
            format!("Failed to recommend frames: {e}"),
        )
    })?;

    let result = retrieve_assets_for_search(library, search_results).await?;
    Ok(result)
}
