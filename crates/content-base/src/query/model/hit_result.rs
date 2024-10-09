use crate::db::model::id::ID;
use crate::db::model::payload::PayloadModel;
use crate::db::model::SelectResultModel;
use crate::db::search::BacktrackResult;
use crate::query::model::SearchType;

#[derive(Debug, Clone)]
pub struct HitResult {
    pub origin_id: ID,
    pub score: f32,
    pub hit_id: Vec<ID>,
    pub payload: PayloadModel,
    pub search_type: SearchType,
    pub result: SelectResultModel,
}

impl From<(BacktrackResult, f32, SearchType, PayloadModel)> for HitResult {
    fn from(
        (bt, score, search_type, payload): (BacktrackResult, f32, SearchType, PayloadModel),
    ) -> Self {
        HitResult {
            origin_id: bt.origin_id,
            score,
            hit_id: bt.hit_id,
            payload,
            result: bt.result,
            search_type,
        }
    }
}

impl HitResult {
    pub fn hit_text(&self, range: Option<(usize, usize)>) -> Option<String> {
        self.result.hit_text(range)
    }
}
