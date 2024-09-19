use crate::db::model::id::ID;
use crate::query::model::full_text::FullTextSearchResult;
use crate::query::model::vector::VectorSearchResult;
use std::collections::HashMap;
use tracing::info;

pub struct Rank;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct RankResult {
    pub id: ID,
}

impl Rank {
    /// 按照分数之和排序
    /// 越大越靠前
    pub fn full_text_rank(
        data: Vec<FullTextSearchResult>,
        drain: Option<usize>,
    ) -> anyhow::Result<Vec<RankResult>> {
        let drain = std::cmp::min(drain.unwrap_or(data.len()), data.len());
        let mut res = data;
        res.sort_by(|a, b| {
            let a_avg_score = a.score.iter().map(|x| x.1).sum::<f32>() / a.score.len() as f32;
            let b_avg_score = b.score.iter().map(|x| x.1).sum::<f32>() / b.score.len() as f32;
            b_avg_score
                .partial_cmp(&a_avg_score)
                .ok_or(std::cmp::Ordering::Equal)
                .unwrap()
        });
        Ok(res
            .drain(..drain)
            .map(|x| RankResult { id: x.id.clone() })
            .collect())
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
                .ok_or(std::cmp::Ordering::Equal)
                .unwrap()
        });
        Ok(res
            .drain(..drain)
            .map(|x| RankResult { id: x.id.clone() })
            .collect())
    }

    pub fn rank(
        (full_text_data, vector_data): (Vec<FullTextSearchResult>, Vec<VectorSearchResult>),
        drain: Option<usize>,
    ) -> anyhow::Result<Vec<RankResult>> {
        let full_text_rank = Rank::full_text_rank(full_text_data, None)?;
        let vector_rank = Rank::vector_rank(vector_data, None)?;

        info!("full_text_rank: {:?}", full_text_rank);
        info!("vector_rank: {:?}", vector_rank);

        let mut rank_result = Rank::rrf(vec![full_text_rank, vector_rank], None);

        info!("rank_result: {:?}", rank_result);

        let drain = std::cmp::min(drain.unwrap_or(rank_result.len()), rank_result.len());

        Ok(rank_result.drain(..drain).collect())
    }
}

trait Rankable {
    fn id(&self) -> String;
    fn from_str(s: &str) -> Self;
}

impl Rankable for RankResult {
    fn id(&self) -> String {
        self.id.id()
    }

    fn from_str(s: &str) -> Self {
        RankResult { id: s.into() }
    }
}

impl Rank {
    fn rrf<T: Rankable>(rankings: Vec<Vec<T>>, k: Option<usize>) -> Vec<T> {
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
            .into_iter()
            .map(|f| T::from_str(f.as_str()))
            .collect()
    }
}

#[cfg(test)]
mod test {
    use crate::db::model::id::ID;
    use crate::query::model::full_text::FullTextSearchResult;
    use crate::query::model::vector::VectorSearchResult;
    use crate::query::rank::{Rank, Rankable};

    #[test]
    fn test_vector_rank() {
        let data = vec![
            VectorSearchResult {
                id: ID::new("1".to_string(), "text"),
                distance: 0.1,
            },
            VectorSearchResult {
                id: ID::new("2".to_string(), "text"),
                distance: 0.2,
            },
            VectorSearchResult {
                id: ID::new("3".to_string(), "text"),
                distance: 0.3,
            },
        ];
        let res = Rank::vector_rank(data, None).unwrap();
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
        let res = Rank::full_text_rank(data, None);
        assert_eq!(res.is_ok(), true);
        let res = res.unwrap();
        assert_eq!(res.len(), 3);
        assert_eq!(res[0].id.id(), "text:3");
        assert_eq!(res[1].id.id(), "text:2");
        assert_eq!(res[2].id.id(), "text:1");
    }

    impl Rankable for String {
        fn id(&self) -> String {
            self.to_string()
        }

        fn from_str(s: &str) -> Self {
            s.to_string()
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
