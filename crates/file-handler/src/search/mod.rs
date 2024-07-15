mod constants;
pub(crate) mod payload;

use self::constants::RETRIEVAL_COUNT;
use ai::{MultiModalEmbeddingModel, TextEmbeddingModel};
use payload::{SearchPayload, SearchRecordType};
use qdrant_client::{
    client::QdrantClient,
    qdrant::{Condition, Filter, PointId, RecommendPoints, ScoredPoint, SearchPoints},
};
use serde_json::json;
use std::{collections::HashMap, sync::Arc};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct RetrievalResult {
    pub file_identifier: String,
    pub timestamp: i32,
    pub record_type: SearchRecordType,
    pub score: f32,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct SearchResult {
    pub file_identifier: String,
    pub start_timestamp: i32,
    pub end_timestamp: i32,
    pub score: f32,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct SearchRequest {
    pub text: String,
    pub record_type: Option<Vec<SearchRecordType>>,
}

pub enum SearchType {
    Frame,
    FrameCaption,
    Transcript,
}

struct ClipRetrievalInfo {
    pub file_identifier: String,
    pub start_timestamp: i64,
    pub end_timestamp: i64,
    pub scores: Vec<f32>,
}

fn group_results_by_asset(
    score_points: &Vec<ScoredPoint>,
    retrieval_results: &mut HashMap<String, Vec<(i64, f32)>>,
) {
    score_points.iter().for_each(|v| {
        let payload = serde_json::from_value::<SearchPayload>(json!(v.payload));
        if let Ok(payload) = payload {
            let target_file_frames = retrieval_results
                .entry(payload.get_file_identifier().to_string())
                .or_insert(vec![]);
            match payload {
                SearchPayload::Frame { timestamp, .. } => {
                    target_file_frames.push((timestamp, v.score + 0.5))
                }
                SearchPayload::FrameCaption { timestamp, .. } => {
                    target_file_frames.push((timestamp, v.score))
                }
                SearchPayload::Transcript {
                    start_timestamp,
                    end_timestamp,
                    ..
                } => {
                    for timestamp in start_timestamp..=end_timestamp {
                        target_file_frames.push((timestamp, v.score))
                    }
                }
            }
        }
    });
}

async fn reorder_final_results(
    retrieval_results: &mut HashMap<String, Vec<(i64, f32)>>,
) -> anyhow::Result<Vec<SearchResult>> {
    // 对于每个视频，对帧进行排序，再切割为片段
    let mut clip_info = HashMap::new();

    retrieval_results.iter().for_each(|(file_id, frames)| {
        let mut frames = frames.clone();
        // 按照 timestamp 排序，注意会有一些时间戳相同的 frame
        frames.sort_by(|a, b| a.0.cmp(&b.0));

        let mut current_clip = 0;
        let mut idx = 0;
        let mut current_frames_score = vec![];
        let mut current_start_timestamp: Option<i64> = None;

        while idx < frames.len() - 1 {
            let timestamp = frames[idx].0;
            current_frames_score.push(frames[idx].1);
            if current_start_timestamp.is_none() {
                current_start_timestamp = Some(timestamp);
            }

            let next_timestamp = frames[idx + 1].0;
            if next_timestamp - timestamp > 1000 {
                let clip_id = format!("{}-{}", file_id, current_clip);
                clip_info.insert(
                    clip_id,
                    ClipRetrievalInfo {
                        file_identifier: file_id.clone(),
                        start_timestamp: current_start_timestamp.unwrap(),
                        end_timestamp: timestamp,
                        scores: current_frames_score.clone(),
                    },
                );
                current_clip += 1;
                current_frames_score = vec![];
                current_start_timestamp = None;
            }

            idx += 1;
        }

        // 处理最后一帧
        let last_frame = frames.last().unwrap(); // 这里可以忽略错误
        current_frames_score.push(last_frame.1);
        if current_start_timestamp.is_none() {
            // 最后一帧是单独的
            current_start_timestamp = Some(last_frame.0);
        }
        let clip_id = format!("{}-{}", file_id, current_clip);
        clip_info.insert(
            clip_id,
            ClipRetrievalInfo {
                file_identifier: file_id.clone(),
                start_timestamp: current_start_timestamp.unwrap(),
                end_timestamp: last_frame.0,
                scores: current_frames_score.clone(),
            },
        );
    });

    let mut result = vec![];

    // 计算每个片段的加权得分
    clip_info.iter_mut().for_each(|(_, info)| {
        let mut score = info
            .scores
            .iter()
            .max_by(|x, y| x.total_cmp(y))
            .unwrap()
            .to_owned();
        // 用匹配到的数量作为 bonus
        // 数量为1 时不加分，增加数量则按照 log 函数增加，超过5个的也不加分
        score += (info.scores.len().min(5) as f32).log(5.0) * 0.15;

        result.push(SearchResult {
            file_identifier: info.file_identifier.clone(),
            start_timestamp: info.start_timestamp as i32,
            end_timestamp: info.end_timestamp as i32,
            score,
        })
    });

    result.sort_by(|a, b| b.score.total_cmp(&a.score));

    Ok(result)
}

/// 实现基于文本的视频召回，召回固定数量，不支持分页
///
/// 实现思路
/// - 根据输入分别生成 CLIP 文本特征和 text-embedding
/// - 数据召回（对于embedding类型，各召回最多`RETRIEVAL_COUNT` 个结果）
///   （frame_score_mapping 是视频帧和得分之间的对应关系 -> HashMap<VIDEO_ID-FRAME_ID, f32>）
///     - 根据 CLIP 文本特征进行图像召回 (以 0.2 为过滤阈值)，得到 frame_score_mapping_1，得分为 cosine similarity + 0.5 (加分数量有待测试)
///     - 根据 text-embedding 进行 caption 和 transcript 召回 (以 0.8 为过滤阈值)，得到 frame_score_mapping_2，得分为 cosine similarity
///     - (Deprecated) 根据文本匹配进行 transcript 召回，得到 frame_score_mapping_3，得分为 0.5 + 0.5 * (query.length / content.length)
/// - 根据上述 frame_score_mapping 首先进行片段切分，得到 clip_frames_score_mapping
///   （clip_frames_score_mapping 是视频片段、视频帧和得分之间的对应关系 -> HaspMap<CLIP_ID, Vec<f32>> ）
/// - 对 clip_frames_score_mapping 中的每个片段计算加权得分，得分规则如下：
///     - clip_score = MAX(Vec<f32>) + lambda * POOL(Vec<f32>)
///     - 其中 MAX 函数负责找到最高得分作为基础得分，POOL 函数负责汇总所有得分，POOL 函数作为额外 bonus
///     - （亟待进一步优化）POOL 取 log_5^(min(5, 召回数量))，lambda 取 0.15
pub async fn handle_search(
    payload: SearchRequest,
    qdrant: Arc<QdrantClient>,
    vision_collection_name: &str,
    language_collection_name: &str,
    multi_modal_embedding: &MultiModalEmbeddingModel,
    text_embedding: &TextEmbeddingModel,
) -> anyhow::Result<Vec<SearchResult>> {
    let clip_text_embedding: TextEmbeddingModel = multi_modal_embedding.into();
    let clip_text_embedding = clip_text_embedding
        .process_single(payload.text.clone())
        .await?;
    let text_model_embedding = text_embedding.process_single(payload.text.clone()).await?;

    let record_types = payload.record_type.unwrap_or(vec![
        SearchRecordType::Frame,
        SearchRecordType::FrameCaption,
        SearchRecordType::Transcript,
    ]);

    // asset => (timestamp, score)
    let mut retrieval_results: HashMap<String, Vec<(i64, f32)>> = HashMap::new();

    for record_type in record_types {
        let req = match record_type {
            SearchRecordType::Frame => SearchPoints {
                collection_name: vision_collection_name.into(),
                vector: clip_text_embedding.clone(),
                limit: 1,
                with_payload: Some(true.into()),
                filter: Some(Filter::all(vec![Condition::matches(
                    "record_type", // TODO maybe this can be better
                    record_type.to_string(),
                )])),
                score_threshold: Some(0.2),
                ..Default::default()
            },
            _ => SearchPoints {
                collection_name: language_collection_name.into(),
                vector: text_model_embedding.clone(),
                limit: RETRIEVAL_COUNT,
                with_payload: Some(true.into()),
                filter: Some(Filter::all(vec![Condition::matches(
                    "record_type", // TODO maybe this can be better
                    record_type.to_string(),
                )])),
                score_threshold: Some(0.8),
                ..Default::default()
            },
        };

        let response = qdrant.search_points(&req).await?;
        let score_points = response.result;
        group_results_by_asset(&score_points, &mut retrieval_results);

        // 对于 transcript 再进行精准匹配
        // if record_type == SearchRecordType::Transcript {
        //     let results = client
        //         .video_transcript()
        //         .find_many(vec![video_transcript::text::contains(payload.text.clone())])
        //         .take(RETRIEVAL_COUNT as i64)
        //         .exec()
        //         .await?;
        //     results.iter().for_each(|v| {
        //         let target_file_frames = retrieval_results
        //             .entry(v.file_identifier.clone())
        //             .or_insert(vec![]);
        //         let score = (v.text.len() as f32) / (payload.text.len() as f32) * 0.5 + 0.5;
        //         for timestamp in v.start_timestamp..=v.end_timestamp {
        //             target_file_frames.push((timestamp as i64, score));
        //         }
        //     });
        // }
    }

    reorder_final_results(&mut retrieval_results).await
}

pub async fn handle_recommend(
    qdrant: Arc<QdrantClient>,
    vision_collection_name: &str,
    asset_object_hash: &str,
    timestamp: i64,
) -> anyhow::Result<Vec<SearchResult>> {
    let payload = SearchPayload::Frame {
        file_identifier: asset_object_hash.to_string(),
        timestamp,
    };
    let point_id: PointId = payload.get_uuid().to_string().into();

    // asset => (timestamp, score)
    let mut recommend_results: HashMap<String, Vec<(i64, f32)>> = HashMap::new();

    let req = RecommendPoints {
        collection_name: vision_collection_name.to_string(),
        positive: vec![point_id],
        limit: RETRIEVAL_COUNT,
        with_payload: Some(true.into()),
        filter: Some(Filter::all(vec![Condition::matches(
            "record_type", // TODO maybe this can be better
            SearchRecordType::Frame.to_string(),
        )])),
        // it's ok to include frames of the same asset
        // filter: Some(Filter::must_not(vec![
        //     Condition::matches("file_identifier", asset_object_hash.to_string()),
        // ])),
        score_threshold: Some(0.2),
        ..Default::default()
    };

    let response = qdrant.recommend(&req).await?;
    let score_points = response.result;
    group_results_by_asset(&score_points, &mut recommend_results);

    reorder_final_results(&mut recommend_results).await
}

#[test_log::test]
fn logarithm_function_test() {
    for i in 1..10 {
        tracing::info!("{}", (i.min(5) as f32).log(5.0));
    }
}
