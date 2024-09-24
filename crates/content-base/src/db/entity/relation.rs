use serde::Deserialize;
use surrealdb::sql::Thing;

use crate::db::model::id::TB;

#[derive(Debug, Deserialize, Clone, Hash, PartialEq, Eq)]
pub struct RelationEntity {
    id: Thing,
    r#in: Thing,
    out: Thing,
}

impl RelationEntity {
    pub fn in_table(&self) -> TB {
        self.r#in.tb.as_str().into()
    }

    pub fn in_id(&self) -> String {
        format!("{}:{}", self.r#in.tb, self.r#in.id.to_raw())
    }

    pub fn out_id(&self) -> String {
        format!("{}:{}", self.out.tb, self.out.id.to_raw())
    }
}
