use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")] // 用于 serialize 了以后写入数据库 assetObject.mediaData 里面的字段名
pub struct RawTextMetadata {
    pub text_count: u64,
}
