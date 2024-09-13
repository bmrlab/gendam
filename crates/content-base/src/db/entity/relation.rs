use serde::Deserialize;
use surrealdb::sql::Thing;

#[derive(Debug, Deserialize, Clone)]
pub struct RelationEntity {
    id: Thing,
    r#in: Thing,
    out: Thing,
}

impl RelationEntity {
    pub fn in_id(&self) -> String {
        format!("{}:{}", self.r#in.tb, self.r#in.id.to_raw())
    }

    pub fn out_id(&self) -> String {
        format!("{}:{}", self.out.tb, self.out.id.to_raw())
    }
}
