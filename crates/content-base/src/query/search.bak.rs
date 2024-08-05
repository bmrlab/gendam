use super::payload::{
    ClipRetrievalInfo, SearchPayload, SearchResult, SearchResultMetadata, VideoSearchResultMetadata,
};
use qdrant_client::qdrant::ScoredPoint;
use serde_json::json;
use std::collections::HashMap;

pub fn group_results_by_asset(
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
                _ => {
                    todo!()
                }
            }
        }
    });
}

pub async fn reorder_final_results(
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
            metadata: SearchResultMetadata::Video(VideoSearchResultMetadata {
                start_timestamp: info.start_timestamp as i32,
                end_timestamp: info.end_timestamp as i32,
            }),
            score,
        })
    });

    result.sort_by(|a, b| b.score.total_cmp(&a.score));

    Ok(result)
}
