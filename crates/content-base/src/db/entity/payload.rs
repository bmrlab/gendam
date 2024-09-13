use serde::Deserialize;
use surrealdb::sql::Thing;

#[derive(Debug, Deserialize)]
pub struct PayloadEntity {
    id: Thing,
    file_identifier: Option<String>,
    url: Option<String>,
}
