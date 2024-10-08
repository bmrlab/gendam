use crate::db::model::id::ID;
use crate::db::search::BacktrackResult;
use crate::query::model::hit_result::HitResult;
use crate::query::model::SearchType;
use crate::query::payload::RetrievalResultData;
use crate::query::rank::Rank;
use crate::utils::extract_highlighted_content;
use crate::{collect_ordered_async_results, ContentBase};
use itertools::Itertools;
use model::SearchModel;
use payload::SearchResultData;
use tracing::debug;

mod data_handler;
mod highlight;
pub mod model;
pub mod payload;
mod rank;
pub mod search;

const RETRIEVAL_COUNT: u64 = 20;

pub struct QueryPayload {
    query: String,
}

impl QueryPayload {
    pub fn new(query: &str) -> Self {
        Self {
            query: query.to_string(),
        }
    }
}

impl ContentBase {
    /// - 文本搜索流程
    ///     1. 获取全文搜索和向量搜索的结果（全文搜索和向量搜索只会搜索文本和图片）
    ///     2. 将上述结果进行 rank
    ///     3. 对上述 rank 的结果进行向上回溯
    ///     4. 填充 payload 信息
    pub async fn query(&self, payload: QueryPayload) -> anyhow::Result<Vec<SearchResultData>> {
        let with_highlight = true;
        // 目前 QueryPayload 只是文本
        match self.query_payload_to_model(payload).await? {
            SearchModel::Text(text) => {
                debug!("search tokens: {:?}", text.tokens.0);

                let full_text_result = self
                    .db
                    .try_read()?
                    .full_text_search(text.tokens.0, with_highlight)
                    .await?;

                debug!("full text result: {full_text_result:?}");

                let hit_fields = full_text_result
                    .iter()
                    .map(|x| extract_highlighted_content(&x.score[0].0))
                    .flatten()
                    .collect::<Vec<String>>();

                debug!("hit fields: {hit_fields:?}");

                let vector_result = self
                    .db
                    .try_read()?
                    .vector_search(text.text_vector, text.vision_vector, None)
                    .await?;

                let rank_result = Rank::rank(
                    (full_text_result.clone(), vector_result),
                    Some(true),
                    Some(10),
                )?;
                debug!("rank result: {rank_result:?}");
                let search_ids: Vec<ID> =
                    rank_result.iter().map(|x| x.id.clone()).unique().collect();
                debug!("search ids: {search_ids:?}");

                let select_by_id_result = self.db.try_read()?.backtrace_by_ids(search_ids).await?;
                debug!("select by id result: {select_by_id_result:?}");
                let hit_result_futures = select_by_id_result
                    .into_iter()
                    .filter_map(|backtrace| {
                        rank_result
                            .iter()
                            .find(|r| r.id.eq(&backtrace.origin_id))
                            .map(|r| (backtrace, r.score, r.search_type.clone()))
                    })
                    .collect::<Vec<(BacktrackResult, f32, SearchType)>>()
                    .into_iter()
                    .map(|(bt, score, search_type)| async move {
                        let payload = self
                            .db
                            .try_read()?
                            .select_payload_by_id(bt.result.id().expect("id not found"))
                            .await?;
                        Ok::<_, anyhow::Error>((bt, score, search_type, payload).into())
                    })
                    .collect::<Vec<_>>();

                let mut hit_result =
                    collect_ordered_async_results!(hit_result_futures, Vec<HitResult>);

                if with_highlight {
                    hit_result = Self::replace_with_highlight(full_text_result, hit_result);
                }
                debug!("hit result: {:#?}", hit_result);

                Ok(hit_result
                    .into_iter()
                    .filter_map(|hit| self.expand_hit_result(hit).ok())
                    .flatten()
                    .collect::<Vec<SearchResultData>>())
            }
            SearchModel::Image(_) => Ok(vec![]),
        }
    }

    /// 实现基于文本特征的基础召回
    pub async fn retrieve(
        &self,
        payload: QueryPayload,
    ) -> anyhow::Result<Vec<RetrievalResultData>> {
        Ok(Vec::new())
    }
}
