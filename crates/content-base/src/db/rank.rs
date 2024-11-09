use super::model::id::ID;
use crate::concat_arrays;
use crate::query::model::{FullTextSearchResult, SearchType, VectorSearchResult};
use std::collections::{HashMap, HashSet};

pub struct Rank;

#[derive(Clone, Debug, PartialEq)]
pub struct RankResult {
    pub id: ID,
    /// 并不是真正的得分，而是排序的依据
    pub score: f32,
    pub search_type: SearchType,
}

#[allow(dead_code)]
pub enum ScoreType {
    Average,
    Maximum,
}

impl Rank {
    /// 按照分数之和排序
    /// 越大越靠前
    pub fn full_text_rank(
        data: &mut Vec<FullTextSearchResult>,
        score_type: ScoreType,
        drain: Option<usize>,
    ) -> anyhow::Result<Vec<RankResult>> {
        let drain = std::cmp::min(drain.unwrap_or(data.len()), data.len());
        let res = data;
        res.sort_by(|a, b| {
            let a_score = Self::calculate_score(a.score.iter().map(|x| x.1).collect(), &score_type);
            let b_score = Self::calculate_score(b.score.iter().map(|x| x.1).collect(), &score_type);
            b_score
                .partial_cmp(&a_score)
                .ok_or(std::cmp::Ordering::Equal)
                .expect(
                    format!(
                        "Failed to compare a_score: {}, b_score: {}",
                        a_score, b_score
                    )
                    .as_str(),
                )
        });
        Ok(res
            .drain(..drain)
            .map(|x| RankResult {
                id: x.id.clone(),
                score: Self::calculate_score(x.score.iter().map(|x| x.1).collect(), &score_type),
                search_type: SearchType::FullText,
            })
            .collect())
    }

    /// 按照 distance 值排序
    /// 越小越靠前
    pub fn vector_rank(
        data: &mut Vec<VectorSearchResult>,
        drain: Option<usize>,
    ) -> anyhow::Result<Vec<RankResult>> {
        let drain = std::cmp::min(drain.unwrap_or(data.len()), data.len());
        let res = data;
        res.sort_by(|a, b| {
            a.distance
                .partial_cmp(&b.distance)
                .ok_or(std::cmp::Ordering::Equal)
                .unwrap()
        });
        Ok(res
            .drain(..drain)
            .map(|x| RankResult {
                id: x.id.clone(),
                score: if x.distance < 0.0 { 0.0 } else { x.distance },
                search_type: SearchType::Vector(x.vector_type),
            })
            .collect())
    }

    pub fn rank(
        (full_text_data, vector_data): (
            &mut Vec<FullTextSearchResult>,
            &mut Vec<VectorSearchResult>,
        ),
        remove_duplicate: bool,
        drain: Option<usize>,
    ) -> anyhow::Result<Vec<RankResult>> {
        let full_text_rank = Rank::full_text_rank(full_text_data, ScoreType::Average, None)?;
        let vector_rank = Rank::vector_rank(vector_data, None)?;

        tracing::debug!("full_text_rank: {:?}", full_text_rank);
        tracing::debug!("vector_rank: {:?}", vector_rank);

        let concat_arrays = concat_arrays!(full_text_rank.clone(), vector_rank.clone()).into_vec();
        let mut rank_result: Vec<RankResult> = Rank::rrf(vec![full_text_rank, vector_rank], None)
            .into_iter()
            .filter_map(|x| {
                concat_arrays
                    .iter()
                    .find(|y| y.id.id_with_table() == x)
                    .map(|r| RankResult {
                        id: ID::from(x.as_str()),
                        score: r.score,
                        search_type: r.search_type.clone(),
                    })
            })
            .collect();

        tracing::debug!("rank_result: {:?}", rank_result);

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

    fn calculate_score(score: Vec<f32>, score_type: &ScoreType) -> f32 {
        match score_type {
            ScoreType::Average => score.iter().sum::<f32>() / score.len() as f32,
            ScoreType::Maximum => score
                .into_iter()
                .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                .unwrap_or(0.0),
        }
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
    fn rrf<T: Rankable>(rankings: Vec<Vec<T>>, k: Option<usize>) -> Vec<String> {
        let mut rrf_scores: HashMap<String, f64> = HashMap::new();
        let k = k.unwrap_or(60);

        for ranking in rankings {
            for (rank, item) in ranking.into_iter().enumerate() {
                let doc_id = item.id();
                let score = rrf_scores.entry(doc_id).or_insert(0.0);
                *score += 1.0 / (k as f64 + rank as f64 + 1.0);
            }
        }

        let mut fused_ranking: Vec<String> = rrf_scores.keys().cloned().collect();
        fused_ranking.sort_by(|a, b| {
            rrf_scores[b]
                .partial_cmp(&rrf_scores[a])
                .unwrap_or(std::cmp::Ordering::Equal)
        });

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
        let mut data = vec![
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
        let res = Rank::vector_rank(&mut data, None).unwrap();
        assert_eq!(res.len(), 3);
        assert_eq!(res[0].id.id(), "1");
        assert_eq!(res[1].id.id(), "2");
        assert_eq!(res[2].id.id(), "3");
    }

    #[test]
    fn test_full_text_rank() {
        let mut data = vec![
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
        let res = Rank::full_text_rank(&mut data, ScoreType::Average, None);
        assert_eq!(res.is_ok(), true);
        let res = res.unwrap();
        assert_eq!(res.len(), 3);
        assert_eq!(res[0].id.id_with_table(), "text:3");
        assert_eq!(res[1].id.id_with_table(), "text:2");
        assert_eq!(res[2].id.id_with_table(), "text:1");

        let mut data = vec![
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
        let res = Rank::full_text_rank(&mut data, ScoreType::Maximum, None);
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
        assert_eq!(vec!["doc2", "doc3", "doc1", "doc5", "doc4"], fused_ranking);
    }
}
