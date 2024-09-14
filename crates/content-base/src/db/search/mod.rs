use anyhow::bail;
use futures::future::join_all;
use tracing::debug;

use crate::{
    db::{constant::SELEC_LIMIT, entity::full_text::FullTextSearchEntity},
    query::model::{
        full_text::{FullTextSearchResult, FULL_TEXT_SEARCH_TABLE},
        vector::{VectorSearchResult, VECTOR_SEARCH_TABLE},
    },
};

use super::{constant::MAX_FULLTEXT_TOKEN, entity::vector::VectorSearchEntity, DB};

mod relation;

// search
impl DB {
    // 🔍 全文搜索实现
    pub async fn full_text_search(
        &self,
        data: Vec<String>,
    ) -> anyhow::Result<Vec<FullTextSearchResult>> {
        if data.is_empty() {
            return Ok(vec![]);
        }
        let data = if data.len() <= MAX_FULLTEXT_TOKEN {
            &data[..]
        } else {
            &data[0..MAX_FULLTEXT_TOKEN]
        };

        let futures = FULL_TEXT_SEARCH_TABLE.iter().map(|table| {
            let param_sql = |data: (usize, &String)| -> (String, String) {
                (
                    format!("search::score({}) AS score_{}", data.0, data.0),
                    format!("{} @{}@ '{}'", table.column_name(), data.0, data.1),
                )
            };

            let (search_scores, where_clauses): (Vec<_>, Vec<_>) =
                data.iter().enumerate().map(param_sql).unzip();

            let sql = format!(
                "SELECT id, {} FROM {} WHERE {} LIMIT {};",
                search_scores.join(", "),
                table.table_name(),
                where_clauses.join(" OR "),
                SELEC_LIMIT
            );
            debug!(
                "full-text search sql on table {}: {sql}",
                table.table_name()
            );

            let data: Vec<String> = data.into_iter().map(|d| d.to_string()).collect();
            async move {
                let text: Vec<FullTextSearchEntity> = self.client.query(&sql).await?.take(0)?;
                Ok::<_, anyhow::Error>(
                    text.iter()
                        .map(|t| t.convert_to_result(&data))
                        .collect::<Vec<_>>(),
                )
            }
        });

        let res: Vec<FullTextSearchResult> = join_all(futures)
            .await
            .into_iter()
            .collect::<anyhow::Result<Vec<_>>>()?
            .into_iter()
            .flatten()
            .collect();
        Ok(res)
    }

    // 🔍 向量搜索实现
    pub async fn vector_search(
        &self,
        data: Vec<f32>,
        range: Option<&str>,
    ) -> anyhow::Result<Vec<VectorSearchResult>> {
        if data.is_empty() {
            bail!("data is empty in vector search");
        }
        let range = range.unwrap_or_else(|| "<|10,40|>");
        let futures = VECTOR_SEARCH_TABLE.map(|v| {
            let data = data.clone();
            async move {
                let mut res = self
                    .client
                    .query(format!("SELECT id, vector::distance::knn() AS distance FROM {} WHERE {} {} $vector ORDER BY distance LIMIT {};", v.table_name(), v.column_name(), range, SELEC_LIMIT))
                    .bind(("vector", data))
                    .await?;
                let res: Vec<VectorSearchEntity> = res.take(0)?;
                Ok::<_, anyhow::Error>(res.iter().map(|d| d.into()).collect::<Vec<VectorSearchResult>>())
            }
        });

        let mut res: Vec<VectorSearchResult> = join_all(futures)
            .await
            .into_iter()
            .collect::<anyhow::Result<Vec<_>>>()?
            .into_iter()
            .flatten()
            .collect();
        res.sort_by(|a, b| {
            a.distance
                .partial_cmp(&b.distance)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        Ok(res)
    }
}
