use itertools::Itertools;

use super::model::id::ID;
use crate::concat_arrays;
use crate::query::model::{FullTextSearchResult, SearchType, VectorSearchResult, VectorSearchType};
use std::collections::{HashMap, HashSet};
pub struct Rank;

#[derive(Clone, Debug, PartialEq)]
pub struct RankResult {
    pub id: ID,
    /// 并不是全文搜索的 score，也不是向量搜索的 distance，是 rff 的得分，只是排序的依据
    pub score: f32,
    /// 真正的得分等辅助信息，格式是 distance:xxx, score:xxx，取决于搜索类型
    pub search_hint: String,
    pub search_type: SearchType,
}

#[allow(dead_code)]
pub enum ScoreType {
    Average,
    Maximum,
}

fn calculate_score(score: Vec<f32>, score_type: &ScoreType) -> f32 {
    match score_type {
        ScoreType::Average => score.iter().sum::<f32>() / score.len() as f32,
        ScoreType::Maximum => score
            .into_iter()
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(0.0),
    }
}

impl Rank {
    /// 按照分数之和排序
    /// 越大越靠前
    pub fn full_text_rank(
        data: Vec<FullTextSearchResult>,
        score_type: ScoreType,
        drain: Option<usize>,
    ) -> anyhow::Result<Vec<RankResult>> {
        let drain = std::cmp::min(drain.unwrap_or(data.len()), data.len());
        let mut res = data;
        res.sort_by(|a, b| {
            let a_score = calculate_score(
                a.score.iter().map(|(_, score)| *score).collect(),
                &score_type,
            );
            let b_score = calculate_score(
                b.score.iter().map(|(_, score)| *score).collect(),
                &score_type,
            );
            b_score
                .partial_cmp(&a_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let rank_results = res
            .drain(..drain)
            .map(|x| {
                let score = calculate_score(
                    x.score.iter().map(|(_, score)| *score).collect(),
                    &score_type,
                );
                let search_hint = x
                    .score
                    .iter()
                    .map(|(_, score)| format!("fulltext.score:{}", score))
                    .collect::<Vec<String>>()
                    .join(",");
                RankResult {
                    id: x.id.clone(),
                    score,
                    search_hint,
                    search_type: SearchType::FullText,
                }
            })
            .collect();
        Ok(rank_results)
    }

    /// 按照 distance 值排序
    /// 越小越靠前
    pub fn vector_rank(
        data: Vec<VectorSearchResult>,
        drain: Option<usize>,
    ) -> anyhow::Result<Vec<RankResult>> {
        let drain = std::cmp::min(drain.unwrap_or(data.len()), data.len());
        let mut res = data;
        res.sort_by(|a, b| {
            a.distance
                .partial_cmp(&b.distance)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        Ok(res
            .drain(..drain)
            .map(|x| {
                let score = if x.distance < 0.0 { 0.0 } else { x.distance };
                let search_hint = match x.vector_type {
                    VectorSearchType::Text => format!("vector.text.distance:{}", x.distance),
                    VectorSearchType::Vision => format!("vector.vision.distance:{}", x.distance),
                };
                RankResult {
                    id: x.id.clone(),
                    score,
                    search_hint,
                    search_type: SearchType::Vector(x.vector_type),
                }
            })
            .collect())
    }

    pub fn rank(
        (full_text_data, vector_data): (Vec<FullTextSearchResult>, Vec<VectorSearchResult>),
        remove_duplicate: bool,
        drain: Option<usize>,
    ) -> anyhow::Result<Vec<RankResult>> {
        let full_text_rank = Rank::full_text_rank(full_text_data, ScoreType::Average, None)?;
        let vector_rank = Rank::vector_rank(vector_data, None)?;
        // tracing::debug!("full_text_rank: {:?}", full_text_rank);
        // tracing::debug!("vector_rank: {:?}", vector_rank);
        let concat_arrays = concat_arrays!(full_text_rank.clone(), vector_rank.clone()).into_vec();
        let mut rank_result: Vec<RankResult> = Rank::rrf(vec![full_text_rank, vector_rank], None)
            .into_iter()
            .filter_map(|(id, score)| {
                let items = concat_arrays
                    .iter()
                    .filter(|y| y.id.id_with_table() == id)
                    .collect_vec();
                if items.len() > 0 {
                    let item = items[0].clone();
                    let search_hint = items
                        .iter()
                        .map(|x| x.search_hint.clone())
                        .collect::<Vec<String>>()
                        .join(",");
                    Some(RankResult {
                        id: item.id,
                        score,
                        search_hint,
                        search_type: item.search_type,
                    })
                } else {
                    None
                }
            })
            .collect();
        // tracing::debug!("rank_result: {:?}", rank_result);
        if remove_duplicate {
            let mut seen = HashSet::new();
            rank_result = rank_result
                .into_iter()
                .filter(|rank| seen.insert(rank.id.clone()))
                .collect();
        }
        let drain = std::cmp::min(drain.unwrap_or(rank_result.len()), rank_result.len());

        Ok(rank_result.drain(..drain).collect())
    }
}

trait Rankable {
    fn id(&self) -> String;
}

impl Rankable for RankResult {
    fn id(&self) -> String {
        self.id.id_with_table()
    }
}

impl Rank {
    fn rrf<T: Rankable>(rankings: Vec<Vec<T>>, k: Option<usize>) -> Vec<(String, f32)> {
        let mut rrf_scores: HashMap<String, f32> = HashMap::new();
        let k = k.unwrap_or(60);

        for ranking in rankings {
            for (rank, item) in ranking.into_iter().enumerate() {
                let doc_id = item.id();
                let score = rrf_scores.entry(doc_id.clone()).or_insert(0.0);
                *score += 1.0 / (k as f32 + rank as f32 + 1.0);
            }
        }

        let mut fused_ranking: Vec<(String, f32)> = rrf_scores
            .iter()
            .map(|(id, &score)| (id.clone(), score))
            .collect();

        fused_ranking.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        fused_ranking
    }
}

#[cfg(test)]
mod test {
    use crate::db::{
        model::id::ID,
        rank::{Rank, Rankable, ScoreType},
    };
    use crate::query::model::{FullTextSearchResult, VectorSearchResult, VectorSearchType};

    #[test]
    fn test_vector_rank() {
        let data = vec![
            VectorSearchResult {
                id: ID::new("1".to_string(), "text"),
                distance: 0.1,
                vector_type: VectorSearchType::Text,
            },
            VectorSearchResult {
                id: ID::new("2".to_string(), "text"),
                distance: 0.2,
                vector_type: VectorSearchType::Text,
            },
            VectorSearchResult {
                id: ID::new("3".to_string(), "text"),
                distance: 0.3,
                vector_type: VectorSearchType::Text,
            },
        ];
        let res = Rank::vector_rank(data.clone(), None).unwrap();
        assert_eq!(res.len(), 3);
        assert_eq!(res[0].id.id(), "1");
        assert_eq!(res[1].id.id(), "2");
        assert_eq!(res[2].id.id(), "3");
    }

    #[test]
    fn test_full_text_rank() {
        let data = vec![
            FullTextSearchResult {
                id: ID::new("1".to_string(), "text"),
                score: vec![
                    ("a".to_string(), 0.1),
                    ("b".to_string(), 0.2),
                    ("bb".to_string(), 0.2),
                    ("bbb".to_string(), 0.2),
                ],
            },
            FullTextSearchResult {
                id: ID::new("2".to_string(), "text"),
                score: vec![("c".to_string(), 0.2), ("d".to_string(), 0.3)],
            },
            FullTextSearchResult {
                id: ID::new("3".to_string(), "text"),
                score: vec![("e".to_string(), 0.3), ("f".to_string(), 0.4)],
            },
        ];
        let res = Rank::full_text_rank(data.clone(), ScoreType::Average, None);
        assert_eq!(res.is_ok(), true);
        let res = res.unwrap();
        assert_eq!(res.len(), 3);
        assert_eq!(res[0].id.id_with_table(), "text:3");
        assert_eq!(res[1].id.id_with_table(), "text:2");
        assert_eq!(res[2].id.id_with_table(), "text:1");

        let data = vec![
            FullTextSearchResult {
                id: ID::new("1".to_string(), "text"),
                score: vec![
                    ("a".to_string(), 0.2),
                    ("b".to_string(), 0.2),
                    ("bb".to_string(), 0.2),
                    ("bbb".to_string(), 0.2),
                ],
            },
            FullTextSearchResult {
                id: ID::new("2".to_string(), "text"),
                score: vec![("c".to_string(), 0.1), ("d".to_string(), 0.8)],
            },
            FullTextSearchResult {
                id: ID::new("3".to_string(), "text"),
                score: vec![("e".to_string(), 0.5), ("f".to_string(), 0.4)],
            },
        ];
        let res = Rank::full_text_rank(data.clone(), ScoreType::Maximum, None);
        assert_eq!(res.is_ok(), true);
        let res = res.unwrap();
        assert_eq!(res[0].id.id_with_table(), "text:2");
        assert_eq!(res[1].id.id_with_table(), "text:3");
        assert_eq!(res[2].id.id_with_table(), "text:1");
    }

    impl Rankable for String {
        fn id(&self) -> String {
            self.to_string()
        }
    }

    #[test]
    fn test_rrt() {
        let ranking1 = vec!["doc1", "doc2", "doc3", "doc4"]
            .iter()
            .map(|x| x.to_string())
            .collect();
        let ranking2 = vec!["doc3", "doc2", "doc1", "doc5"]
            .iter()
            .map(|x| x.to_string())
            .collect();
        let ranking3 = vec!["doc2", "doc3", "doc5", "doc1"]
            .iter()
            .map(|x| x.to_string())
            .collect();
        let rankings = vec![ranking1, ranking2, ranking3];

        let fused_ranking = Rank::rrf(rankings, None);
        assert_eq!(
            vec!["doc2", "doc3", "doc1", "doc5", "doc4"],
            fused_ranking
                .iter()
                .map(|x| x.0.clone())
                .collect::<Vec<String>>()
        );
    }
}
