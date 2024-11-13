mod full_text_search;
mod test;
mod vector_search;
use crate::{
    db::{model::id::ID, rank::Rank, DB},
    query::{
        model::{FullTextSearchResult, SearchModel, SearchType, VectorSearchType},
        payload::{
            audio::{AudioIndexMetadata, AudioSliceType},
            image::ImageIndexMetadata,
            raw_text::{RawTextChunkType, RawTextIndexMetadata},
            video::{VideoIndexMetadata, VideoSliceType},
            web_page::{WebPageChunkType, WebPageIndexMetadata},
            ContentIndexMetadata, ContentQueryHitReason, ContentQueryResult,
        },
    },
    utils::extract_highlighted_content,
};
use serde::Deserialize;
use std::convert::Into;

use super::rank::RankResult;

impl DB {
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

                let hit_words = full_text_results
                    .iter()
                    .map(|x| extract_highlighted_content(&x.score[0].0))
                    .flatten()
                    .collect::<Vec<String>>();
                tracing::debug!("hit words {hit_words:?}");

                let vector_results = self
                    .vector_search(text.text_vector, text.vision_vector)
                    .await?;
                tracing::debug!("{} found in vector search", vector_results.len());

                // 需要复制一下 full_text_results 和 vector_results，rank 方法会清空这两个 vec
                let rank_result = Rank::rank(
                    (full_text_results.clone(), vector_results.clone()),
                    false,
                    Some(max_count),
                )?;
                tracing::debug!("{} results after rank", rank_result.len());

                let query_results = self
                    .lookup_payload_by_ids(&rank_result, &full_text_results)
                    .await?;
                tracing::debug!("{} results after lookup", query_results.len());

                Ok(query_results)
            }
            _ => unimplemented!(),
        }
    }

    async fn lookup_payload_by_ids(
        &self,
        rank_results: &Vec<RankResult>,
        full_text_results: &Vec<FullTextSearchResult>,
    ) -> anyhow::Result<Vec<ContentQueryResult>> {
        let full_text_results_map = full_text_results
            .into_iter()
            .map(|r| (r.id.clone(), r))
            .collect::<std::collections::HashMap<_, _>>();
        let rank_results_map = rank_results
            .into_iter()
            .map(|r| (r.id.clone(), r))
            .collect::<std::collections::HashMap<_, _>>();
        let ids = rank_results_map.keys().collect::<Vec<_>>();
        let things = ids
            .into_iter()
            .map(|id| surrealdb::sql::Thing::from(id))
            .collect::<Vec<_>>();
        let mut res = self
            .client
            .query(PAYLOAD_LOOKUP_SQL)
            .bind(("ids", things))
            .await?;
        let res_image: Vec<PayloadLookupResult> = res.take(0)?;
        let res_text: Vec<PayloadLookupResult> = res.take(1)?;
        let res = Vec::new()
            .into_iter()
            .chain(res_image.into_iter())
            .chain(res_text.into_iter())
            .collect::<Vec<PayloadLookupResult>>();
        let mut query_results: Vec<ContentQueryResult> = Vec::new();
        for record in res {
            let id: ID = record.id.into();
            let rank_result = rank_results_map
                .get(&id)
                .ok_or_else(|| anyhow::anyhow!("Missing rank result"))?;
            let highlight = match full_text_results_map.get(&id) {
                Some(r) => r.score[0].0.clone(),
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
                        AudioSliceType::Transcript => {
                            ContentQueryHitReason::TranscriptMatch(highlight)
                        }
                    },
                    ContentIndexMetadata::Image(_) => {
                        ContentQueryHitReason::CaptionMatch(highlight)
                    }
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
            });
        }
        Ok(query_results)
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
            reference_text: prompt,
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
            reference_text: prompt,
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
            reference_text: data,
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
            reference_text: data,
            asset_id: id,
            frame: None,
            file_identifier: ->with[0].out.file_identifier
        }
    }) AS A
FROM text
WHERE id in $ids).A;
"#;
