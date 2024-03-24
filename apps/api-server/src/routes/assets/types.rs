use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Deserialize, Type, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FilePathRequestPayload {
    pub id: i32,
    pub is_dir: bool,
    pub path: String,
    pub name: String,
}

#[derive(Serialize, Type, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MediaDataQueryResult {
    pub id: i32,
    pub width: i32,
    pub height: i32,
    pub duration: i32,
    pub bit_rate: i32,
    pub size: i32,
}

#[derive(Serialize, Type, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AssetObjectQueryResult {
    pub id: i32,
    pub hash: String,
    pub media_data: Option<MediaDataQueryResult>,
}

#[derive(Serialize, Type, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FilePathQueryResult {
    pub id: i32,
    pub name: String,
    pub materialized_path: String,
    pub is_dir: bool,
    pub asset_object: Option<AssetObjectQueryResult>,
    pub created_at: String,
    pub updated_at: String,
}
