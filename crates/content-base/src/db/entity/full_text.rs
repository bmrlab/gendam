use serde::Deserialize;
use std::collections::HashMap;
use surrealdb::sql::Thing;

use crate::query::model::FullTextSearchResult;

#[derive(Debug, Deserialize)]
pub(crate) struct FullTextSearchEntity {
    id: Thing,
    #[serde(flatten)]
    scores: HashMap<String, f32>,
}

impl FullTextSearchEntity {
    pub fn convert_to_result(&self, words: &Vec<String>) -> FullTextSearchResult {
        let score = words
            .iter()
            .enumerate()
            .map(|(i, word)| {
                (
                    word.clone(),
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

#[derive(Debug, Deserialize)]
pub(crate) struct FullTextWithHighlightSearchEntity {
    id: Thing,
    score: f32,
    highlight: String,
}

impl From<FullTextWithHighlightSearchEntity> for FullTextSearchResult {
    fn from(value: FullTextWithHighlightSearchEntity) -> Self {
        FullTextSearchResult {
            id: value.id.clone().into(),
            score: vec![(value.highlight, value.score)],
        }
    }
}
