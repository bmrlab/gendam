use super::payload::{
    audio::AudioSearchMetadata, raw_text::RawTextSearchMetadata,
    video::merge_results_with_time_duration, web_page::WebPageSearchMetadata, SearchPayload,
    SearchResultData,
};
use crate::query::payload::{video::VideoSearchMetadata, SearchMetadata};
use qdrant_client::qdrant::ScoredPoint;
use serde_json::json;
use std::collections::HashMap;

pub fn group_results_by_asset(
    scored_points: &[ScoredPoint],
    retrieval_results: &mut HashMap<String, Vec<(SearchPayload, f32)>>,
) {
    scored_points.iter().for_each(|v| {
        if let Ok(payload) = serde_json::from_value::<SearchPayload>(json!(v.payload)) {
            let target_file_results = retrieval_results
                .entry(payload.file_identifier().to_string())
                .or_insert(vec![]);
            target_file_results.push((payload, v.score))
        }
    });
}

pub fn reorder_final_results(
    retrieval_results: &mut HashMap<String, Vec<(SearchPayload, f32)>>,
) -> anyhow::Result<Vec<SearchResultData>> {
    let mut reordered_results = vec![];

    retrieval_results.iter().for_each(|(file_id, results)| {
        let result = &results.first().expect("results should not be empty").0;
        // 同一个文件对应的 SearchPayload 应该都是同样的类型
        match result.metadata {
            SearchMetadata::Video(_) => {
                let mut results: Vec<(VideoSearchMetadata, f32)> = results
                    .iter()
                    .filter_map(|v| {
                        let (payload, score) = v;
                        match payload.metadata.clone().try_into() {
                            Ok(metadata) => Some((metadata, *score)),
                            _ => None,
                        }
                    })
                    .collect();

                let results = merge_results_with_time_duration(
                    &mut results,
                    |items| {
                        let start_timestamp = items
                            .iter()
                            .map(|v| v.start_timestamp)
                            .min()
                            .expect("should have min");
                        let end_timestamp = items
                            .iter()
                            .map(|v| v.end_timestamp)
                            .max()
                            .expect("should have max");

                        VideoSearchMetadata {
                            start_timestamp,
                            end_timestamp,
                        }
                    },
                    |current, last| current.start_timestamp - last.end_timestamp > 1000,
                );

                results.into_iter().for_each(|v| {
                    reordered_results.push(SearchResultData {
                        file_identifier: file_id.clone(),
                        score: v.1,
                        metadata: v.0.into(),
                    })
                });
            }
            SearchMetadata::Audio(_) => {
                let mut results: Vec<(AudioSearchMetadata, f32)> = results
                    .iter()
                    .filter_map(|v| {
                        let (payload, score) = v;
                        match payload.metadata.clone().try_into() {
                            Ok(metadata) => Some((metadata, *score)),
                            _ => None,
                        }
                    })
                    .collect();

                let results = merge_results_with_time_duration(
                    &mut results,
                    |items| {
                        let start_timestamp = items
                            .iter()
                            .map(|v| v.start_timestamp)
                            .min()
                            .expect("should have min");
                        let end_timestamp = items
                            .iter()
                            .map(|v| v.end_timestamp)
                            .max()
                            .expect("should have max");

                        AudioSearchMetadata {
                            start_timestamp,
                            end_timestamp,
                        }
                    },
                    |current, last| current.start_timestamp - last.end_timestamp > 1000,
                );

                results.into_iter().for_each(|v| {
                    reordered_results.push(SearchResultData {
                        file_identifier: file_id.clone(),
                        score: v.1,
                        metadata: v.0.into(),
                    })
                });
            }
            SearchMetadata::Image(_) => {
                results.iter().for_each(|v| {
                    reordered_results.push(SearchResultData {
                        file_identifier: file_id.clone(),
                        score: v.1,
                        metadata: v.0.metadata.clone(),
                    })
                });
            }
            SearchMetadata::RawText(_) => {
                let mut results: Vec<(RawTextSearchMetadata, f32)> = results
                    .iter()
                    .filter_map(|v| {
                        let (payload, score) = v;
                        match payload.metadata.clone().try_into() {
                            Ok(metadata) => Some((metadata, *score)),
                            _ => None,
                        }
                    })
                    .collect();

                let results = merge_results_with_time_duration(
                    &mut results,
                    |items| {
                        let index = items
                            .iter()
                            .map(|v| v.index)
                            .min()
                            .expect("should have min");

                        RawTextSearchMetadata { index }
                    },
                    |current, last| current.index - last.index > 1,
                );

                results.into_iter().for_each(|v| {
                    reordered_results.push(SearchResultData {
                        file_identifier: file_id.clone(),
                        score: v.1,
                        metadata: v.0.into(),
                    })
                });
            }
            SearchMetadata::WebPage(_) => {
                let mut results: Vec<(WebPageSearchMetadata, f32)> = results
                    .iter()
                    .filter_map(|v| {
                        let (payload, score) = v;
                        match payload.metadata.clone().try_into() {
                            Ok(metadata) => Some((metadata, *score)),
                            _ => None,
                        }
                    })
                    .collect();

                let results = merge_results_with_time_duration(
                    &mut results,
                    |items| {
                        let index = items
                            .iter()
                            .map(|v| v.index)
                            .min()
                            .expect("should have min");

                        WebPageSearchMetadata { index }
                    },
                    |current, last| current.index - last.index > 1,
                );

                results.into_iter().for_each(|v| {
                    reordered_results.push(SearchResultData {
                        file_identifier: file_id.clone(),
                        score: v.1,
                        metadata: v.0.into(),
                    })
                });
            }
        }
    });

    reordered_results.sort_by(|a, b| b.score.total_cmp(&a.score));

    Ok(reordered_results)
}
