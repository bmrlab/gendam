use std::collections::HashMap;

use self::search_payload::SearchRecordType;
use faiss::Index;
use prisma_lib::{new_client_with_url, video_frame};
use tracing::debug;
pub(self) mod audio;
pub mod embedding;
pub mod index;
pub mod search_payload;
pub mod video;

// TODO constants should be extracted into global config
pub const EMBEDDING_DIM: usize = 512;

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
    pub record_type: SearchRecordType,
    pub limit: Option<u64>,
}

pub enum SearchType {
    Frame,
    FrameCaption,
    Transcript,
}

pub async fn handle_search(
    payload: SearchRequest,
    resources_dir: impl AsRef<std::path::Path>,
    local_data_dir: impl AsRef<std::path::Path>,
    db_url: impl AsRef<std::path::Path>,
) -> anyhow::Result<Vec<SearchResult>> {
    let client =
        new_client_with_url(&format!("file:{}", db_url.as_ref().to_str().unwrap())).await?;

    let clip_model =
        embedding::clip::CLIP::new(embedding::clip::model::CLIPModel::ViTB32, &resources_dir)
            .await?;

    let embedding = clip_model.get_text_embedding(&payload.text).await?;
    let embedding: Vec<f32> = embedding.iter().map(|&x| x).collect();

    debug!("embedding: {:?}", embedding);

    let mut index = faiss::read_index(
        local_data_dir
            .as_ref()
            .join("index")
            .join(payload.record_type.index_name())
            .to_str()
            .unwrap(),
    )?
    .into_id_map()?;

    debug!("index vector count: {}", index.ntotal());
    debug!("index dimension: {}", index.d());

    let results = index.search(embedding.as_slice(), payload.limit.unwrap_or(10) as usize)?;

    tracing::debug!("labels: {:?}", results.labels);
    tracing::debug!("distances: {:?}", results.distances);

    let mut id_order_mapping = HashMap::new();

    let ids = results
        .labels
        .iter()
        .zip(results.distances.iter())
        .enumerate()
        .map(|(order, (id, &distance))| {
            let id = id.get().unwrap() as i32;
            id_order_mapping.insert(id, (order, distance));

            id
        })
        .collect();

    // find results from prisma using labels
    match payload.record_type {
        SearchRecordType::Frame => {
            let mut results = client
                .video_frame()
                .find_many(vec![video_frame::WhereParam::Id(
                    prisma_lib::read_filters::IntFilter::InVec(ids),
                )])
                .exec()
                .await?;

            results.sort_by(|a, b| {
                id_order_mapping
                    .get(&a.id)
                    .unwrap()
                    .0
                    .cmp(&id_order_mapping.get(&b.id).unwrap().0)
            });

            Ok(results
                .iter()
                .map(|v| SearchResult {
                    file_identifier: v.file_identifier.clone(),
                    start_timestamp: v.timestamp,
                    end_timestamp: v.timestamp,
                    score: id_order_mapping.get(&v.id).unwrap().1,
                })
                .collect())
        }
        SearchRecordType::FrameCaption => {
            todo!()
        }
        SearchRecordType::Transcript => {
            todo!()
        }
    }
}

#[test_log::test(tokio::test)]
async fn test_handle_search() {
    let results = handle_search(
        SearchRequest {
            text: "a photo of a girl".into(),
            record_type: SearchRecordType::Frame,
            limit: None,
        },
        "/Users/zhuo/dev/bmrlab/tauri-dam-test-playground/target/debug/resources",
        "/Users/zhuo/Library/Application Support/cc.musedam.local",
        "/Users/zhuo/Library/Application Support/cc.musedam.local/dev.db",
    )
    .await;

    debug!("results: {:?}", results);
}
