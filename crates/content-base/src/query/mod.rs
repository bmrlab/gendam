mod data_handler;
pub mod model;
pub mod payload;
use crate::query::model::hit_result::HitResult;
use crate::query::payload::RetrievalResultData;
use crate::ContentBase;
use payload::SearchResultData;

const RETRIEVAL_COUNT: u64 = 20;
const MAX_RANK_COUNT: usize = 10;

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
    pub async fn query(
        &self,
        payload: QueryPayload,
        max_count: Option<usize>,
    ) -> anyhow::Result<Vec<SearchResultData>> {
        let max_count = max_count.unwrap_or(MAX_RANK_COUNT);
        let with_highlight = true;

        let hit_result = self
            .db
            .try_read()?
            .search(
                self.query_payload_to_model(payload).await?,
                with_highlight,
                max_count,
            )
            .await?;

        Ok(hit_result
            .into_iter()
            .filter_map(|hit| self.expand_hit_result(hit).ok())
            .flatten()
            .collect::<Vec<SearchResultData>>())
    }

    pub async fn retrieve(
        &self,
        payload: QueryPayload,
    ) -> anyhow::Result<Vec<RetrievalResultData>> {
        Ok(self
            .query(payload, Some(RETRIEVAL_COUNT as usize))
            .await?
            .into_iter()
            .map(|data| data.into())
            .collect::<Vec<RetrievalResultData>>())
    }
}
