use serde::Deserialize;
use std::collections::HashMap;
use surrealdb::sql::Thing;

use crate::query::model::full_text::FullTextSearchResult;

#[derive(Debug, Deserialize)]
pub(crate) struct FullTextSearchEntity {
    id: Thing,
    #[serde(flatten)]
    scores: HashMap<String, f32>,
}

impl FullTextSearchEntity {
    pub fn convert_to_result(&self, data: &Vec<String>) -> FullTextSearchResult {
        let score = data
            .iter()
            .enumerate()
            .map(|(i, d)| {
                (
                    d.clone(),
                    self.scores
                        .get(&format!("score_{}", i))
                        .unwrap_or(&0.0)
                        .clone(),
                )
            })
            .collect();
        FullTextSearchResult {
            id: self.id.clone().into(),
            score,
        }
    }
}
