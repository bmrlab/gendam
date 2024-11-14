use crate::{
    check_db_error_from_resp,
    db::{
        model::{image::ImageModel, text::TextModel},
        DB,
    },
    query::model::{VectorSearchResult, VectorSearchType},
};
use futures::future::join_all;
use serde::Deserialize;
use std::convert::Into;

const VECTOR_QUERY_LIMIT: usize = 100;

// TODO: vision 和 text 向量现在采用了不同的命中范围，这个要继续调整
const VISION_VECTOR_RANGE: &str = "<|2,20|>";
const TEXT_VECTOR_RANGE: &str = "<|10,40|>";

#[derive(Debug, Deserialize)]
pub(crate) struct VectorSearchEntity {
    pub id: surrealdb::sql::Thing,
    pub distance: f32,
}

fn vector_query_statement(table: &str, vector_column: &str, range: &str) -> String {
    format!(
        r#"
SELECT
    id,
    vector::distance::knn() AS distance
FROM {table}
WHERE {vector_column} {range} $vector_value
ORDER BY distance
LIMIT {limit};
"#,
        table = table,
        vector_column = vector_column,
        range = range,
        limit = VECTOR_QUERY_LIMIT
    )
}

/// 组装 (table, column) 的元组数组，给后面使用，只搜索 Image 和 Text 基础对象，然后再回溯关联的对象
fn vector_search_columns() -> Vec<(&'static str, &'static str, &'static VectorSearchType)> {
    let params = vec![
        (
            ImageModel::table(),
            ImageModel::text_embedding_columns(),
            &VectorSearchType::Text,
        ), // 向量搜索 Image 的 prompt 文本特征
        (
            TextModel::table(),
            TextModel::text_embedding_columns(),
            &VectorSearchType::Text,
        ), // 向量搜索 Text 的文本特征
        (
            ImageModel::table(),
            ImageModel::vision_embedding_columns(),
            &VectorSearchType::Vision,
        ), // 向量搜索 Image 的图像特征
    ]
    .into_iter()
    .map(|(table, columns, vector_type)| {
        columns
            .into_iter()
            .map(|column| (table, column, vector_type))
            .collect::<Vec<(&str, &str, &VectorSearchType)>>()
    })
    .flatten()
    .collect::<Vec<(&str, &str, &VectorSearchType)>>();
    params
}

impl DB {
    /// 🔍 vector search
    ///
    /// if not vision_vector, please input text_vector
    pub async fn vector_search(
        &self,
        text_vector: Vec<f32>,
        vision_vector: Vec<f32>,
    ) -> anyhow::Result<Vec<VectorSearchResult>> {
        if text_vector.is_empty() || vision_vector.is_empty() {
            anyhow::bail!("data is empty in vector search");
        }

        // 组装 (table, column, vector_value) 的元组数组，给后面使用
        let params = vector_search_columns();
        let futures = params.into_iter().map(|(table, column, vector_type)| {
            let (vector_value, range) = match vector_type {
                VectorSearchType::Text => (text_vector.clone(), TEXT_VECTOR_RANGE),
                VectorSearchType::Vision => (vision_vector.clone(), VISION_VECTOR_RANGE),
            };
            let query_statement = vector_query_statement(table, column, range);
            async move {
                let mut res = self
                    .client
                    .query(query_statement)
                    .bind(("vector_value", vector_value))
                    .await?;
                check_db_error_from_resp!(res).map_err(|errors_map| {
                    tracing::error!("vector_search errors: {errors_map:?}");
                    anyhow::anyhow!("vector_search errors: {errors_map:?}")
                })?;
                let res: Vec<VectorSearchEntity> = res.take(0)?;
                Ok::<_, anyhow::Error>(
                    res.iter()
                        .map(|d| VectorSearchResult {
                            id: d.id.clone().into(),
                            distance: d.distance,
                            vector_type: vector_type.clone(),
                        })
                        .collect::<Vec<VectorSearchResult>>(),
                )
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
