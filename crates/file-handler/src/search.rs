use crate::search_payload::{SearchPayload, SearchRecordType};
use ai::{clip::{CLIPInput, CLIP}, BatchHandler};
use prisma_lib::{video_frame, video_frame_caption, video_transcript, PrismaClient};
use qdrant_client::{
    client::QdrantClient,
    qdrant::{Condition, Filter, SearchPoints},
};
use serde_json::json;
use std::{collections::HashMap, sync::Arc};

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
    pub limit: Option<u64>,
    pub skip: Option<u64>,
}

pub enum SearchType {
    Frame,
    FrameCaption,
    Transcript,
}

pub async fn handle_search(
    payload: SearchRequest,
    client: Arc<PrismaClient>,
    qdrant: Arc<QdrantClient>,
    clip: BatchHandler<CLIP>,
) -> anyhow::Result<Vec<SearchResult>> {
    let embeddings = clip.process(vec![CLIPInput::Text(payload.text.clone())]).await?;
    let embedding = embeddings.into_iter().next().expect("embedding not found")?;
    let embedding: Vec<f32> = embedding.iter().map(|&x| x).collect();

    let record_types = payload.record_type.unwrap_or(vec![
        SearchRecordType::Frame,
        SearchRecordType::FrameCaption,
        SearchRecordType::Transcript,
    ]);
    let mut search_results = vec![];

    for record_type in record_types {
        let search_result = qdrant
            .search_points(&SearchPoints {
                collection_name: vector_db::DEFAULT_COLLECTION_NAME.into(),
                vector: embedding.clone(),
                limit: payload.limit.unwrap_or(10),
                offset: payload.skip,
                with_payload: Some(true.into()),
                filter: Some(Filter::all(vec![Condition::matches(
                    "record_type", // TODO maybe this can be better
                    record_type.to_string(),
                )])),
                ..Default::default()
            })
            .await?;

        let mut id_score_mapping = HashMap::new();

        let ids = search_result
            .result
            .iter()
            .filter_map(|v| {
                let payload = serde_json::from_value::<SearchPayload>(json!(v.payload));

                match payload {
                    Ok(payload) => {
                        id_score_mapping.insert(payload.get_id() as i32, v.score);
                        Some(payload.get_id() as i32)
                    }
                    _ => None,
                }
            })
            .collect();

        match record_type {
            SearchRecordType::Frame => {
                let results = client
                    .video_frame()
                    .find_many(vec![video_frame::WhereParam::Id(
                        prisma_lib::read_filters::IntFilter::InVec(ids),
                    )])
                    .exec()
                    .await?;

                results.iter().for_each(|v| {
                    search_results.push(SearchResult {
                        file_identifier: v.file_identifier.clone(),
                        start_timestamp: v.timestamp,
                        end_timestamp: v.timestamp,
                        score: *id_score_mapping.get(&v.id).unwrap(),
                    })
                });
            }
            SearchRecordType::FrameCaption => {
                let results = client
                    .video_frame_caption()
                    .find_many(vec![video_frame_caption::WhereParam::Id(
                        prisma_lib::read_filters::IntFilter::InVec(ids),
                    )])
                    .with(video_frame_caption::frame::fetch())
                    .exec()
                    .await?;

                results.iter().for_each(|v| {
                    // TODO: 这里忽略了找不到的 frame，其实不应该有找不到的情况，需要优化下
                    let frame = match v.frame.as_ref() {
                        Some(frame) => frame,
                        None => return,
                    };

                    search_results.push(SearchResult {
                        file_identifier: frame.file_identifier.clone(),
                        start_timestamp: frame.timestamp,
                        end_timestamp: frame.timestamp,
                        score: *id_score_mapping.get(&v.id).unwrap(),
                    })
                });
            }
            SearchRecordType::Transcript => {
                let results = client
                    .video_transcript()
                    .find_many(vec![video_transcript::WhereParam::Id(
                        prisma_lib::read_filters::IntFilter::InVec(ids),
                    )])
                    .exec()
                    .await?;

                results.iter().for_each(|v| {
                    search_results.push(SearchResult {
                        file_identifier: v.file_identifier.clone(),
                        start_timestamp: v.start_timestamp,
                        end_timestamp: v.end_timestamp,
                        score: *id_score_mapping.get(&v.id).unwrap(),
                    })
                });
            }
        }
    }

    // order results by score
    search_results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
    Ok(search_results.into_iter().collect())
}
