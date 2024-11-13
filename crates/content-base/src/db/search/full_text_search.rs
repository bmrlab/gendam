use crate::{
    check_db_error_from_resp,
    constant::HIGHLIGHT_MARK,
    db::{
        model::{image::ImageModel, text::TextModel},
        DB,
    },
    query::model::FullTextSearchResult,
};
use futures::future::join_all;
use serde::Deserialize;
use std::collections::HashMap;
use std::convert::Into;
use surrealdb::sql::Thing;

pub const MAX_FULLTEXT_TOKEN: usize = 100;
pub const FULL_TEXT_QUERY_LIMIT: usize = 100;

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

// 使用 $query var 就不需要在两边加引号了，sueeral 会自动处理类型，加了引号就搜索不出来了
fn full_text_query_statement(table: &str, column: &str) -> String {
    format!(
        r#"
SELECT
    id,
    search::score(0) as score,
    search::highlight('{mark_left}', '{mark_right}', 0) AS highlight
FROM {table}
WHERE {column} @0@ $query
LIMIT {limit};"#,
        table = table,
        column = column,
        mark_left = HIGHLIGHT_MARK.0,
        mark_right = HIGHLIGHT_MARK.1,
        limit = FULL_TEXT_QUERY_LIMIT
    )
}

/// 组装 (table, column) 的元组数组，给后面使用
/// 全文搜索 Image 的 prompt 和 Text 的 data，然后再回溯关联的对象
fn full_text_search_columns() -> Vec<(&'static str, &'static str)> {
    let params = vec![
        (ImageModel::table(), ImageModel::full_text_columns()),
        (TextModel::table(), TextModel::full_text_columns()),
    ]
    .into_iter()
    .map(|(table, columns)| {
        columns
            .into_iter()
            .map(|column| (table, column))
            .collect::<Vec<(&str, &str)>>()
    })
    .flatten()
    .collect::<Vec<(&str, &str)>>();
    params
}

impl DB {
    pub async fn full_text_search(
        &self,
        data: Vec<String>,
        with_highlight: bool,
    ) -> anyhow::Result<Vec<FullTextSearchResult>> {
        Ok(if with_highlight {
            self.full_text_search_with_highlight(data).await?
        } else {
            self._full_text_search(data).await?
        })
    }

    /// 🔍 full text search
    /// - 对每个分词进行全文搜索
    /// - 分词之间使用 OR 连接
    /// - 缺点是高亮结果是分散的
    /// SELECT id, search::score(0) AS score_0, search::score(1) AS score_1, search::score(2) AS score_2
    /// FROM {table}
    /// WHERE {column} @0@ '$word_0' OR {column} @1@ '$word_1' OR {column} @2@ '$word_2'
    /// LIMIT {limit};
    async fn _full_text_search(
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

        // 组装 (table, column) 的元组数组，给后面使用
        let columns = full_text_search_columns();
        let futures = columns.into_iter().map(|(table, column)| {
            let (search_scores, where_clauses): (Vec<String>, Vec<String>) = data
                .iter()
                .enumerate()
                .map(|(index, word)| {
                    let search_score = format!("search::score({}) AS score_{}", index, index);
                    let where_clause = format!("{} @{}@ '{}'", column, index, word);
                    (search_score, where_clause)
                })
                .unzip();

            let sql = format!(
                "SELECT id, {select} FROM {table} WHERE {where_clauses} LIMIT {limit};",
                select = search_scores.join(", "),
                table = table,
                where_clauses = where_clauses.join(" OR "),
                limit = FULL_TEXT_QUERY_LIMIT
            );

            let data: Vec<String> = data.into_iter().map(|d| d.to_string()).collect();
            async move {
                let text: Vec<FullTextSearchEntity> = self.client.query(sql).await?.take(0)?;
                Ok::<_, anyhow::Error>(
                    text.iter()
                        .map(|t| t.convert_to_result(&data))
                        .collect::<Vec<_>>(),
                )
            }
        });

        Ok(join_all(futures)
            .await
            .into_iter()
            .collect::<anyhow::Result<Vec<_>>>()?
            .into_iter()
            .flatten()
            .collect())
    }

    /// 全文搜索并高亮
    /// - 将整个搜索结果丢进去，然后返回高亮结果
    /// - 分词之间的结果是 AND 连接
    /// - 缺点是无法直接确定命中了哪个分词
    ///    - 可以通过正则 <b></b> 来确定关键词
    pub async fn full_text_search_with_highlight(
        &self,
        data: Vec<String>,
    ) -> anyhow::Result<Vec<FullTextSearchResult>> {
        if data.is_empty() {
            return Ok(vec![]);
        }

        let query = data.join(" ");
        // 组装 (table, column) 的元组数组，给后面使用
        let columns = full_text_search_columns();
        let futures = columns.into_iter().map(|(table, column)| {
            let query_statement = full_text_query_statement(table, column);
            let query = query.clone();
            async move {
                let mut resp = self
                    .client
                    .query(query_statement)
                    .bind(("query", query))
                    .await?;
                check_db_error_from_resp!(resp).map_err(|errors_map| {
                    tracing::error!("full_text_search_with_highlight errors: {errors_map:?}");
                    anyhow::anyhow!("full_text_search_with_highlight errors: {errors_map:?}")
                })?;
                let text: Vec<FullTextWithHighlightSearchEntity> = resp.take(0)?;
                Ok::<_, anyhow::Error>(
                    text.into_iter()
                        .map(Into::into)
                        .collect::<Vec<FullTextSearchResult>>(),
                )
            }
        });

        Ok(join_all(futures)
            .await
            .into_iter()
            .collect::<anyhow::Result<Vec<_>>>()?
            .into_iter()
            .flatten()
            .collect())
    }
}
