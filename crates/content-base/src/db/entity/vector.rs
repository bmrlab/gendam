use serde::Deserialize;
use surrealdb::sql::Thing;

#[derive(Debug, Deserialize)]
pub(crate) struct VectorSearchEntity {
    pub id: Thing,
    pub distance: f32,
}
