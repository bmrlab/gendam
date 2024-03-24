use serde::Deserialize;
use specta::Type;

#[derive(Deserialize, Type, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FilePathRequestPayload {
    pub id: i32,
    pub is_dir: bool,
    pub path: String,
    pub name: String,
}
