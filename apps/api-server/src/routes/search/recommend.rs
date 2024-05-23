use super::search::{retrieve_assets_for_search, SearchResultPayload};
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
    qdrant_info: &QdrantServerInfo,
    asset_object_hash: &str,
    timestamp: i32,
) -> Result<Vec<SearchResultPayload>, rspc::Error> {
    let search_results = file_handler::search::handle_recommend(
        library.qdrant_client(),
        qdrant_info.vision_collection.name.as_ref(),
        asset_object_hash,
        timestamp as i64,
    )
    .await
    .map_err(|e| {
        tracing::error!("Failed to recommend frames: {e}");
        rspc::Error::new(
            rspc::ErrorCode::InternalServerError,
            format!("Failed to recommend frames: {e}"),
        )
    })?;

    let result = retrieve_assets_for_search(library, search_results).await?;
    Ok(result)
}
