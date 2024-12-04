mod full_text_search;
mod test;
mod vector_search;
use serde::Deserialize;
use std::collections::HashMap;
use std::convert::Into;

use super::rank::RankResult;
use crate::{
    db::{model::id::ID, rank::Rank, DB},
    query::{
        model::{SearchModel, SearchType, VectorSearchType},
        payload::{
            audio::{AudioIndexMetadata, AudioSliceType},
            image::ImageIndexMetadata,
            raw_text::{RawTextChunkType, RawTextIndexMetadata},
            video::{VideoIndexMetadata, VideoSliceType},
            web_page::{WebPageChunkType, WebPageIndexMetadata},
            ContentIndexMetadata, ContentQueryHitReason, ContentQueryResult,
        },
    },
    // utils::extract_highlighted_content,
};

async fn lookup_assets_by_image_text_ids<T: surrealdb::Connection>(
    surrealdb_client: &surrealdb::Surreal<T>,
    rank_results_map: &HashMap<ID, RankResult>,
    full_text_highlight_map: &HashMap<ID, String>,
) -> anyhow::Result<Vec<ContentQueryResult>> {
    let ids = rank_results_map.keys().collect::<Vec<_>>();
    let things = ids
        .into_iter()
        .map(|id| surrealdb::sql::Thing::from(id))
        .collect::<Vec<_>>();
    // tracing::debug!(ids=?things, "look up assets by image and text ids");
    let mut res = surrealdb_client
        .query(PAYLOAD_LOOKUP_SQL)
        .bind(("ids", things))
        .await?;

    // tracing::debug!(response=?res, "look up assets by image and text ids");
    let res_image: Vec<PayloadLookupResult> = res.take(0)?;
    let res_text: Vec<PayloadLookupResult> = res.take(1)?;
    let res = Vec::new()
        .into_iter()
        .chain(res_image.into_iter())
        .chain(res_text.into_iter())
        .collect::<Vec<PayloadLookupResult>>();

    let mut query_results: Vec<ContentQueryResult> = Vec::new();
    // text 和 image 可能对应到同样的视频片段，那么，视频的分数是不是应该增加？
    // 也就是 query_results 里面是会有重复的 file_identifier 的
    // https://github.com/bmrlab/gendam/issues/105#issuecomment-2509669785
    // TODO: 需要合并一下重复的 frame

    for record in res {
        let id: ID = record.id.into();
        let rank_result = rank_results_map
            .get(&id)
            .ok_or_else(|| anyhow::anyhow!("Missing rank result"))?
            .to_owned();
        let highlight = match full_text_highlight_map.get(&id) {
            Some(v) => v.clone(),
            None => "".to_string(),
        };
        let reference_text = record.reference_text.clone();
        let metadata = match (record.asset_id.tb.as_str(), &record.segment) {
            ("image", None) => {
                let metadata = ImageIndexMetadata { data: 0 };
                ContentIndexMetadata::Image(metadata)
            }
            ("audio", Some(SegmentLookup::AudioFrame(segment))) => {
                let metadata = AudioIndexMetadata {
                    slice_type: AudioSliceType::Transcript,
                    start_timestamp: segment.start_timestamp,
                    end_timestamp: segment.end_timestamp,
                };
                ContentIndexMetadata::Audio(metadata)
            }
            ("video", Some(SegmentLookup::ImageFrame(segment))) => {
                let metadata = VideoIndexMetadata {
                    slice_type: VideoSliceType::Visual,
                    start_timestamp: segment.start_timestamp,
                    end_timestamp: segment.end_timestamp,
                };
                ContentIndexMetadata::Video(metadata)
            }
            ("video", Some(SegmentLookup::AudioFrame(segment))) => {
                let metadata = VideoIndexMetadata {
                    slice_type: VideoSliceType::Audio,
                    start_timestamp: segment.start_timestamp,
                    end_timestamp: segment.end_timestamp,
                };
                ContentIndexMetadata::Video(metadata)
            }
            ("document", Some(SegmentLookup::Page(segment))) => {
                let metadata = RawTextIndexMetadata {
                    chunk_type: RawTextChunkType::Content,
                    start_index: segment.start_index,
                    end_index: segment.end_index,
                };
                ContentIndexMetadata::RawText(metadata)
            }
            ("web", Some(SegmentLookup::Page(segment))) => {
                let metadata = WebPageIndexMetadata {
                    chunk_type: WebPageChunkType::Content,
                    start_index: segment.start_index,
                    end_index: segment.end_index,
                };
                ContentIndexMetadata::WebPage(metadata)
            }
            _ => {
                anyhow::bail!(
                    "unexpected asset type {:?} or segment type {:?}",
                    &record.asset_id,
                    &record.segment
                );
            }
        };
        let hit_reasone = match rank_result.search_type {
            SearchType::FullText => match &metadata {
                ContentIndexMetadata::Video(metadata) => match metadata.slice_type {
                    VideoSliceType::Visual => ContentQueryHitReason::CaptionMatch(highlight),
                    VideoSliceType::Audio => ContentQueryHitReason::TranscriptMatch(highlight),
                },
                ContentIndexMetadata::Audio(metadata) => match metadata.slice_type {
                    AudioSliceType::Transcript => ContentQueryHitReason::TranscriptMatch(highlight),
                },
                ContentIndexMetadata::Image(_) => ContentQueryHitReason::CaptionMatch(highlight),
                _ => ContentQueryHitReason::TextMatch(highlight),
            },
            SearchType::Vector(VectorSearchType::Text) => match &metadata {
                ContentIndexMetadata::Video(metadata) => match metadata.slice_type {
                    VideoSliceType::Visual => {
                        ContentQueryHitReason::SemanticCaptionMatch(reference_text)
                    }
                    VideoSliceType::Audio => {
                        ContentQueryHitReason::SemanticTranscriptMatch(reference_text)
                    }
                },
                ContentIndexMetadata::Audio(metadata) => match metadata.slice_type {
                    AudioSliceType::Transcript => {
                        ContentQueryHitReason::SemanticTranscriptMatch(reference_text)
                    }
                },
                ContentIndexMetadata::Image(_) => {
                    ContentQueryHitReason::SemanticCaptionMatch(reference_text)
                }
                _ => ContentQueryHitReason::SemanticTextMatch(reference_text),
            },
            SearchType::Vector(VectorSearchType::Vision) => ContentQueryHitReason::VisionMatch,
        };
        query_results.push(ContentQueryResult {
            file_identifier: record.file_identifier,
            score: rank_result.score,
            metadata,
            hit_reason: Some(hit_reasone),
            reference_content: Some(record.reference_text),
            search_hint: rank_result.search_hint.clone(),
        });
    }

    Ok(query_results)
}

async fn merge_frames(query_results: &mut Vec<ContentQueryResult>) -> anyhow::Result<()> {
    type Meta = ContentIndexMetadata;
    query_results.sort_by(|a, b| {
        match (
            &a.file_identifier.cmp(&b.file_identifier),
            &a.metadata,
            &b.metadata,
        ) {
            (std::cmp::Ordering::Equal, Meta::Video(a_meta), Meta::Video(b_meta)) => a_meta
                .start_timestamp
                .cmp(&b_meta.start_timestamp)
                .then(a_meta.end_timestamp.cmp(&b_meta.end_timestamp)),
            (std::cmp::Ordering::Equal, Meta::Audio(a_meta), Meta::Audio(b_meta)) => a_meta
                .start_timestamp
                .cmp(&b_meta.start_timestamp)
                .then(a_meta.end_timestamp.cmp(&b_meta.end_timestamp)),
            (std::cmp::Ordering::Equal, Meta::WebPage(a_meta), Meta::WebPage(b_meta)) => a_meta
                .start_index
                .cmp(&b_meta.start_index)
                .then(a_meta.end_index.cmp(&b_meta.end_index)),
            (std::cmp::Ordering::Equal, Meta::RawText(a_meta), Meta::RawText(b_meta)) => a_meta
                .start_index
                .cmp(&b_meta.start_index)
                .then(a_meta.end_index.cmp(&b_meta.end_index)),
            (ordering, _, _) => *ordering,
        }
    });

    // 用于存储合并后的结果
    let mut merged_results: Vec<ContentQueryResult> = Vec::new();
    let mut current_result: Option<ContentQueryResult> = None;

    for mut result in query_results.drain(..) {
        match current_result {
            None => {
                current_result = Some(result);
            }
            Some(mut curr) => {
                if curr.file_identifier == result.file_identifier {
                    // 检查是否有重叠并合并
                    if merge_if_overlapping(&mut curr, &mut result) {
                        current_result = Some(curr);
                    } else {
                        // 如果没有重叠，保存当前结果并开始新的
                        merged_results.push(curr);
                        current_result = Some(result);
                    }
                } else {
                    // 不同文件，保存当前结果并开始新的
                    merged_results.push(curr);
                    current_result = Some(result);
                }
            }
        }
    }

    // 处理最后一个结果
    if let Some(last_result) = current_result {
        merged_results.push(last_result);
    }

    // 更新原始 vector
    *query_results = merged_results;

    Ok(())
}

fn merge_if_overlapping(a: &mut ContentQueryResult, b: &mut ContentQueryResult) -> bool {
    let (a_start, a_end, b_start, b_end) = match (&a.metadata, &b.metadata) {
        (ContentIndexMetadata::Video(a_meta), ContentIndexMetadata::Video(b_meta)) => (
            a_meta.start_timestamp,
            a_meta.end_timestamp,
            b_meta.start_timestamp,
            b_meta.end_timestamp,
        ),
        (ContentIndexMetadata::Audio(a_meta), ContentIndexMetadata::Audio(b_meta)) => (
            a_meta.start_timestamp,
            a_meta.end_timestamp,
            b_meta.start_timestamp,
            b_meta.end_timestamp,
        ),
        (ContentIndexMetadata::WebPage(a_meta), ContentIndexMetadata::WebPage(b_meta)) => (
            a_meta.start_index as i64,
            a_meta.end_index as i64,
            b_meta.start_index as i64,
            b_meta.end_index as i64,
        ),
        (ContentIndexMetadata::RawText(a_meta), ContentIndexMetadata::RawText(b_meta)) => (
            a_meta.start_index as i64,
            a_meta.end_index as i64,
            b_meta.start_index as i64,
            b_meta.end_index as i64,
        ),
        _ => return false,
    };

    // 检查是否重叠，1s以内的算重叠
    if b_start <= a_end + 1000 {
        // 更新 metadata 中的 start/end
        match &mut a.metadata {
            ContentIndexMetadata::Video(meta) => {
                meta.start_timestamp = a_start.min(b_start);
                meta.end_timestamp = a_end.max(b_end);
            }
            ContentIndexMetadata::Audio(meta) => {
                meta.start_timestamp = a_start.min(b_start);
                meta.end_timestamp = a_end.max(b_end);
            }
            ContentIndexMetadata::WebPage(meta) => {
                meta.start_index = a_start.min(b_start) as usize;
                meta.end_index = a_end.max(b_end) as usize;
            }
            ContentIndexMetadata::RawText(meta) => {
                meta.start_index = a_start.min(b_start) as usize;
                meta.end_index = a_end.max(b_end) as usize;
            }
            _ => {}
        }

        // 取较高的分数
        a.score = a.score.max(b.score);

        // 合并 search_hint
        if !b.search_hint.is_empty() {
            if !a.search_hint.is_empty() {
                a.search_hint.push_str("; ");
            }
            a.search_hint.push_str(&b.search_hint);
        }

        true
    } else {
        false
    }
}

impl DB {
    #[tracing::instrument(err(Debug), skip_all)]
    pub async fn search(
        &self,
        data: SearchModel,
        with_highlight: bool,
        max_count: usize,
    ) -> anyhow::Result<Vec<ContentQueryResult>> {
        match data {
            SearchModel::Text(text) => {
                tracing::debug!("search tokens: {:?}", text.tokens.0);

                let full_text_results =
                    self.full_text_search(text.tokens.0, with_highlight).await?;
                tracing::debug!("{} found in full text search", full_text_results.len());

                // let hit_words = full_text_results.iter().map(|x| extract_highlighted_content(&x.score[0].0)).flatten().collect::<Vec<String>>();
                // tracing::debug!("hit words {hit_words:?}");

                let vector_results = self
                    .vector_search(text.text_embedding, text.vision_embedding)
                    .await?;
                tracing::debug!("{} found in vector search", vector_results.len());

                // 需要复制一下 full_text_results 和 vector_results，rank 方法会清空这两个 vec
                let rank_result = Rank::rank(
                    (full_text_results.clone(), vector_results.clone()),
                    false,
                    Some(max_count),
                )?;
                tracing::debug!("{} results after rank", rank_result.len());

                let full_text_highlight_map = full_text_results
                    .iter()
                    .map(|r| match r.score.get(0) {
                        Some((highlight, _score)) => (r.id.clone(), highlight.clone()),
                        None => (r.id.clone(), "".to_string()),
                    })
                    .collect::<HashMap<_, _>>();
                let rank_results_map = rank_result
                    .into_iter()
                    .map(|r| (r.id.clone(), r))
                    .collect::<HashMap<_, _>>();
                let mut query_results = lookup_assets_by_image_text_ids(
                    &self.client,
                    &rank_results_map,
                    &full_text_highlight_map,
                )
                .await?;
                tracing::debug!("{} results after lookup", query_results.len());

                merge_frames(&mut query_results).await?;

                // 最后需要排序一下因为 query_results 是按照 id 的顺序返回的
                query_results.sort_by(|a, b| {
                    b.score
                        .partial_cmp(&a.score)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });

                Ok(query_results)
            }
            _ => unimplemented!(),
        }
    }
}

/// 方便 deserialize 的结构体，只限于 PAYLOAD_LOOKUP_SQL 返回结果临时使用
#[derive(Debug, Deserialize)]
struct PayloadLookupResult {
    pub id: surrealdb::sql::Thing,
    pub reference_text: String,
    pub asset_id: surrealdb::sql::Thing,
    pub segment: Option<SegmentLookup>,
    pub file_identifier: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case", tag = "table", content = "value")]
enum SegmentLookup {
    AudioFrame(FrameLookup),
    ImageFrame(FrameLookup),
    Page(PageLookup),
}
#[derive(Debug, Deserialize)]
struct FrameLookup {
    // pub id: surrealdb::sql::Thing,
    pub start_timestamp: i64,
    pub end_timestamp: i64,
}

#[derive(Debug, Deserialize)]
struct PageLookup {
    // pub id: surrealdb::sql::Thing,
    pub start_index: usize,
    pub end_index: usize,
}

const PAYLOAD_LOOKUP_SQL: &'static str = r#"
(SELECT
    (IF <-contains {
        {
            id: id,
            reference_text: caption,
            asset_id: <-contains[0].in<-contains[0].in,
            segment: {
                table: record::tb(<-contains[0].in),
                value: <-contains[0].in.*
            },
            file_identifier: <-contains[0].in<-contains[0].in->with[0].out.file_identifier
        }
    } ELSE {
        {
            id: id,
            reference_text: caption,
            asset_id: id,
            frame: None,
            file_identifier: ->with[0].out.file_identifier
        }
    }) AS A
FROM image
WHERE id in $ids).A;
(SELECT
    (IF <-contains {
        {
            id: id,
            reference_text: content,
            asset_id: <-contains[0].in<-contains[0].in,
            segment: {
                table: record::tb(<-contains[0].in),
                value: <-contains[0].in.*
            },
            file_identifier: <-contains[0].in<-contains[0].in->with[0].out.file_identifier
        }
    } ELSE {
        {
            id: id,
            reference_text: content,
            asset_id: id,
            frame: None,
            file_identifier: ->with[0].out.file_identifier
        }
    }) AS A
FROM text
WHERE id in $ids).A;
"#;
