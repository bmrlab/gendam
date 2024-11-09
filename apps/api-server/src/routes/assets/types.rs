use crate::validators;
use content_metadata::ContentMetadata;
use prisma_lib::{asset_object, data_location, file_handler_task, file_path};
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Deserialize, Type, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FilePathRequestPayload {
    pub id: i32,
    pub is_dir: bool,
    #[serde(deserialize_with = "validators::materialized_path_string")]
    pub materialized_path: String,
    #[serde(deserialize_with = "validators::path_name_string")]
    pub name: String,
}

#[derive(Serialize, Deserialize, Type, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AssetObjectWithMediaData {
    #[serde(rename = "id")]
    pub id: i32,
    #[serde(rename = "hash")]
    pub hash: String,
    #[serde(rename = "size")]
    pub size: i32,
    #[serde(rename = "mimeType")]
    pub mime_type: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at:
        ::prisma_client_rust::chrono::DateTime<::prisma_client_rust::chrono::FixedOffset>,
    #[serde(rename = "updatedAt")]
    pub updated_at:
        ::prisma_client_rust::chrono::DateTime<::prisma_client_rust::chrono::FixedOffset>,
    #[serde(rename = "mediaData")]
    pub media_data: Option<ContentMetadata>,
    #[serde(rename = "filePaths")]
    #[specta(skip)]
    pub file_paths: Option<Vec<file_path::Data>>,
    #[serde(rename = "tasks")]
    #[specta(skip)]
    pub tasks: Option<Vec<file_handler_task::Data>>,
    #[serde(rename = "dataLocations")]
    #[specta(skip)]
    pub data_locations: Option<Vec<data_location::Data>>,
}

#[derive(Serialize, Deserialize, Type, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FilePathWithAssetObjectData {
    #[serde(rename = "id")]
    pub id: i32,
    #[serde(rename = "isDir")]
    pub is_dir: bool,
    #[serde(rename = "materializedPath")]
    pub materialized_path: String,
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "description")]
    pub description: Option<String>,
    #[serde(rename = "assetObjectId")]
    pub asset_object_id: Option<i32>,
    #[serde(
        rename = "assetObject",
        default,
        skip_serializing_if = "Option::is_none",
        with = "prisma_client_rust::serde::double_option"
    )]
    pub asset_object: Option<Option<AssetObjectWithMediaData>>,
    #[serde(rename = "createdAt")]
    pub created_at:
        ::prisma_client_rust::chrono::DateTime<::prisma_client_rust::chrono::FixedOffset>,
    #[serde(rename = "updatedAt")]
    pub updated_at:
        ::prisma_client_rust::chrono::DateTime<::prisma_client_rust::chrono::FixedOffset>,
}

impl From<asset_object::Data> for AssetObjectWithMediaData {
    // 在这里实现了从数据库中读取 media_data 字符串并解析为 `ContentMetadata`
    fn from(value: asset_object::Data) -> Self {
        let media_data = match value.media_data {
            Some(v) => {
                let content_metadata: Result<ContentMetadata, _> = serde_json::from_str(&v);
                content_metadata
                    .map_err(|e| {
                        tracing::error!("Failed to parse media_data from AssetObject: {:?}", e);
                    })
                    .ok()
            }
            _ => None,
        };

        Self {
            id: value.id,
            hash: value.hash,
            size: value.size,
            mime_type: value.mime_type,
            created_at: value.created_at,
            updated_at: value.updated_at,
            media_data,
            file_paths: value.file_paths,
            tasks: value.tasks,
            data_locations: value.data_locations,
        }
    }
}

impl From<file_path::Data> for FilePathWithAssetObjectData {
    fn from(value: file_path::Data) -> Self {
        Self {
            id: value.id,
            is_dir: value.is_dir,
            materialized_path: value.materialized_path,
            name: value.name,
            description: value.description,
            asset_object_id: value.asset_object_id,
            asset_object: value.asset_object.map(|v| v.map(|t| (*t).into())),
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
