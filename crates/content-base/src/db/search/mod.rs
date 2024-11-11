mod backtrace;
mod full_text_search;
mod relation;
mod test;
mod vector_search;
pub use self::backtrace::BacktrackResult;
use crate::{
    collect_ordered_async_results,
    db::{model::id::ID, rank::Rank, utils::replace_with_highlight, DB},
    query::model::{HitResult, SearchModel, SearchType},
    utils::extract_highlighted_content,
};
use itertools::Itertools;
use std::convert::Into;

impl DB {
    pub async fn search(
        &self,
        data: SearchModel,
        with_highlight: bool,
        max_count: usize,
    ) -> anyhow::Result<Vec<HitResult>> {
        match data {
            SearchModel::Text(text) => {
                tracing::debug!("search tokens: {:?}", text.tokens.0);

                let full_text_results =
                    self.full_text_search(text.tokens.0, with_highlight).await?;
                tracing::debug!("{} found in full text search", full_text_results.len());

                let hit_words = full_text_results
                    .iter()
                    .map(|x| extract_highlighted_content(&x.score[0].0))
                    .flatten()
                    .collect::<Vec<String>>();
                tracing::debug!("hit words {hit_words:?}");

                let vector_results = self
                    .vector_search(text.text_vector, text.vision_vector)
                    .await?;
                tracing::debug!("{} found in vector search", vector_results.len());

                // 需要复制一下 full_text_results 和 vector_results，rank 方法会清空这两个 vec
                let rank_result = Rank::rank(
                    (full_text_results.clone(), vector_results.clone()),
                    false,
                    Some(max_count),
                )?;
                tracing::debug!("{} results after rank", rank_result.len(),);

                let backtrace_results = {
                    // 从 text 和 image 表回溯到关联的实体的结果，视频、音频、文档、网页、等
                    let search_ids: Vec<ID> =
                        rank_result.iter().map(|x| x.id.clone()).unique().collect();
                    self.backtrace_by_ids(search_ids).await?
                };
                tracing::debug!("{} backtrace results", backtrace_results.len());

                let mut hit_results = {
                    let futures = backtrace_results
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
                                .select_payload_by_id(bt.result.id().expect("id not found"))
                                .await?;
                            Ok::<_, anyhow::Error>((bt, score, search_type, payload).into())
                        })
                        .collect::<Vec<_>>();
                    collect_ordered_async_results!(futures, Vec<HitResult>)
                };

                if with_highlight {
                    hit_results = replace_with_highlight(full_text_results, hit_results);
                }

                tracing::debug!("{} final hit results", hit_results.len());
                Ok(hit_results)
            }
            _ => unimplemented!(),
        }
    }
}
