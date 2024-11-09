use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "rspc", derive(specta::Type))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")] // 用于 serialize 了以后写入数据库 assetObject.mediaData 里面的字段名
pub struct ImageMetadata {
    pub width: u32,
    pub height: u32,
    pub color: String,
}
