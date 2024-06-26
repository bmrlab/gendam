use serde::Deserialize;
use specta::Type;
use crate::validators;

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
