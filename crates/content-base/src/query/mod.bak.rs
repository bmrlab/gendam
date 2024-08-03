pub mod payload;
pub mod search;

use std::collections::HashMap;
use crate::ContentBase;
use ai::TextEmbeddingModel;
use payload::{SearchPayload, SearchRecordType, SearchResult, VideoRAGReference};
use qdrant_client::qdrant::{
    Condition, Filter, PointId, RecommendPointsBuilder, SearchPointsBuilder,
};
use search::{group_results_by_asset, reorder_final_results};
use serde_json::json;

pub struct QueryPayload {
    query: String,
    record_type: Option<Vec<SearchRecordType>>,
}

impl QueryPayload {
    pub fn new(query: &str) -> Self {
        Self {
            query: query.to_string(),
            record_type: None,
        }
    }

    pub fn with_record_type(&mut self, record_type: &[SearchRecordType]) {
        self.record_type = Some(record_type.to_vec());
    }
}

pub struct RecommendVideoFramePayload {
    file_identifier: String,
    timestamp: i64,
}

impl RecommendVideoFramePayload {
    pub fn new(file_identifier: &str, timestamp: i64) -> Self {
        Self {
            file_identifier: file_identifier.to_string(),
            timestamp,
        }
    }
}

pub struct VideoRAGPayload {
    query: String,
}

impl VideoRAGPayload {
    pub fn new(query: &str) -> Self {
        Self {
            query: query.to_string(),
        }
    }
}

const RETRIEVAL_COUNT: u64 = 20;

impl ContentBase {
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
    pub async fn query(&self, payload: QueryPayload) -> anyhow::Result<Vec<SearchResult>> {
        let text_embedding = self.ctx.text_embedding()?.0;
        let multi_modal_embedding: TextEmbeddingModel = self.ctx.multi_modal_embedding()?.0.into();
        let vision_collection_name = self.ctx.vision_collection_name();
        let language_collection_name = self.ctx.language_collection_name();

        let clip_text_embedding = multi_modal_embedding
            .process_single(payload.query.clone())
            .await?;
        let text_model_embedding = text_embedding.process_single(payload.query.clone()).await?;

        let record_types = payload.record_type.unwrap_or(vec![
            SearchRecordType::Frame,
            SearchRecordType::FrameCaption,
            SearchRecordType::Transcript,
        ]);

        // asset => (timestamp, score)
        let mut retrieval_results: HashMap<String, Vec<(i64, f32)>> = HashMap::new();

        for record_type in record_types {
            let search_points_builder = match record_type {
                SearchRecordType::Frame => {
                    SearchPointsBuilder::new(vision_collection_name, clip_text_embedding.clone(), 1)
                        .score_threshold(0.2)
                }
                _ => SearchPointsBuilder::new(
                    language_collection_name,
                    text_model_embedding.clone(),
                    RETRIEVAL_COUNT,
                )
                .score_threshold(0.8),
            };

            let search_points_builder = search_points_builder
                .filter(Filter::all(vec![Condition::matches(
                    "record_type",
                    record_type.to_string(),
                )]))
                .with_payload(true);

            let response = self
                .ctx
                .qdrant()
                .search_points(search_points_builder)
                .await?;
            let score_points = response.result;
            group_results_by_asset(&score_points, &mut retrieval_results);
        }

        reorder_final_results(&mut retrieval_results).await
    }

    pub async fn recommend_video_frame(
        &self,
        payload: RecommendVideoFramePayload,
    ) -> anyhow::Result<Vec<SearchResult>> {
        let payload = SearchPayload::Frame {
            file_identifier: payload.file_identifier.clone(),
            timestamp: payload.timestamp.clone(),
        };
        let point_id: PointId = payload.get_uuid().to_string().into();

        // asset => (timestamp, score)
        let mut recommend_results: HashMap<String, Vec<(i64, f32)>> = HashMap::new();

        let response = self
            .ctx
            .qdrant()
            .recommend(
                RecommendPointsBuilder::new(self.ctx.vision_collection_name(), RETRIEVAL_COUNT)
                    .add_positive(point_id)
                    .filter(Filter::all(vec![Condition::matches(
                        "record_type",
                        SearchRecordType::Frame.to_string(),
                    )]))
                    .with_payload(true)
                    .score_threshold(0.2),
            )
            .await?;
        let score_points = response.result;
        group_results_by_asset(&score_points, &mut recommend_results);

        reorder_final_results(&mut recommend_results).await
    }

    pub async fn retrieval_video(
        &self,
        payload: VideoRAGPayload,
    ) -> anyhow::Result<Vec<VideoRAGReference>> {
        let text_embedding = self
            .ctx
            .text_embedding()?
            .0
            .process_single(payload.query.clone())
            .await?;

        let response = self
            .ctx
            .qdrant()
            .search_points(
                SearchPointsBuilder::new(self.ctx.language_collection_name(), text_embedding, 5)
                    .filter(Filter::all(vec![Condition::matches(
                        "record_type",
                        SearchRecordType::TranscriptChunkSummarization.to_string(),
                    )]))
                    .with_payload(true),
            )
            .await?;
        let scored_points = response.result;

        Ok(scored_points
            .into_iter()
            .filter_map(|v| {
                let payload = serde_json::from_value::<SearchPayload>(json!(v.payload));
                if let Ok(SearchPayload::TranscriptChunkSummarization {
                    file_identifier,
                    start_timestamp,
                    end_timestamp,
                }) = payload
                {
                    Some(VideoRAGReference {
                        file_identifier: file_identifier.to_string(),
                        chunk_start_timestamp: start_timestamp as i32,
                        chunk_end_timestamp: end_timestamp as i32,
                        score: v.score,
                    })
                } else {
                    None
                }
            })
            .collect())
    }
}
