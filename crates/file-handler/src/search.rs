use crate::index::{self, EmbeddingIndex};
use content_library::Library;
use prisma_lib::{video_frame, video_frame_caption, video_transcript, PrismaClient};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;
use tracing::debug;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum SearchRecordType {
    Frame,
    FrameCaption,
    Transcript,
}

impl SearchRecordType {
    pub fn index_name(&self) -> &str {
        match self {
            SearchRecordType::Frame => index::VIDEO_FRAME_INDEX_NAME,
            SearchRecordType::FrameCaption => index::VIDEO_FRAME_CAPTION_INDEX_NAME,
            SearchRecordType::Transcript => index::VIDEO_TRANSCRIPT_INDEX_NAME,
        }
    }
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
    pub limit: Option<usize>,
}

pub enum SearchType {
    Frame,
    FrameCaption,
    Transcript,
}

pub async fn handle_search(
    payload: SearchRequest,
    resources_dir: impl AsRef<std::path::Path>,
    library: Library,
    client: Arc<RwLock<PrismaClient>>,
) -> anyhow::Result<Vec<SearchResult>> {
    let clip_model =
        ai::clip::CLIP::new(ai::clip::model::CLIPModel::ViTB32, &resources_dir).await?;

    let embedding = clip_model.get_text_embedding(&payload.text).await?;
    let embedding: Vec<f32> = embedding.iter().map(|&x| x).collect();

    debug!("embedding of search text: {:?}", embedding);

    let record_types = payload.record_type.unwrap_or(vec![
        SearchRecordType::Frame,
        SearchRecordType::FrameCaption,
        SearchRecordType::Transcript,
    ]);
    let mut search_results = vec![];

    for record_type in record_types {
        let index = EmbeddingIndex::new(
            &library.index_dir,
            record_type.index_name(),
            None,
        )?;

        let limit = payload.limit.unwrap_or(10);
        let results = index.search(embedding.clone(), limit).await?;

        let mut id_distance_mapping = HashMap::new();

        let ids = results
            .labels
            .iter()
            .zip(results.distances.iter())
            .filter(|(id, _)| id.is_some())
            .map(|(id, &distance)| {
                let id = id.get().unwrap() as i32;
                id_distance_mapping.insert(id, distance);
                id
            })
            .collect();

        let client = client.read().await;

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
                        score: *id_distance_mapping.get(&v.id).unwrap(),
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
                        score: *id_distance_mapping.get(&v.id).unwrap(),
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
                        score: *id_distance_mapping.get(&v.id).unwrap(),
                    })
                });
            }
        }
    }

    // order results by score
    search_results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
    Ok(search_results
        .into_iter()
        .take(payload.limit.unwrap_or(10))
        .collect())
}

#[test_log::test(tokio::test)]
async fn test_handle_search() {
    let local_data_dir =
        std::path::Path::new("/Users/zhuo/Library/Application Support/cc.musedam.local")
            .to_path_buf();
    let library = content_library::create_library_with_title(&local_data_dir, "dev test library").await;
    let library = content_library::load_library(&local_data_dir, &library.id);
    let client = prisma_lib::new_client_with_url(&library.db_url)
        .await
        .expect("failed to create prisma client");
    client._db_push().await.expect("failed to push db"); // apply migrations
    let client = Arc::new(RwLock::new(client));
    let results = handle_search(
        SearchRequest {
            text: "a photo of a girl".into(),
            record_type: None,
            limit: None,
        },
        "/Users/zhuo/dev/bmrlab/tauri-dam-test-playground/target/debug/resources",
        library,
        client,
    )
    .await;

    debug!("results: {:?}", results);
}
