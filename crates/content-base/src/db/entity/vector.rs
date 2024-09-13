use serde::Deserialize;
use surrealdb::sql::Thing;

use crate::query::model::vector::VectorSearchResult;

#[derive(Debug, Deserialize)]
pub(crate) struct VectorSearchEntity {
    id: Thing,
    distance: f32,
}

impl From<&VectorSearchEntity> for VectorSearchResult {
    fn from(entity: &VectorSearchEntity) -> Self {
        VectorSearchResult {
            id: entity.id.clone().into(),
            distance: entity.distance,
        }
    }
}
