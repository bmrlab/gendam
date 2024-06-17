use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Serialize, Deserialize, Type, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct S3Config {
    bucket: String,
    endpoint: String,
    access_key_id: String,
    secret_access_key: String,
}

impl S3Config {
    pub fn new(
        bucket: String,
        endpoint: String,
        access_key_id: String,
        secret_access_key: String,
    ) -> Self {
        Self {
            bucket,
            endpoint,
            access_key_id,
            secret_access_key,
        }
    }
}
