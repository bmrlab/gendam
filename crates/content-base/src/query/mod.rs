use crate::ContentBase;
use ai::TextEmbeddingModel;
use payload::{RetrievalResultData, SearchPayload, SearchResultData};
use qdrant_client::qdrant::SearchPointsBuilder;
use search::{group_results_by_asset, reorder_final_results};
use serde_json::json;
use std::collections::HashMap;

pub mod payload;
pub mod search;

const RETRIEVAL_COUNT: u64 = 20;

pub struct QueryPayload {
    query: String,
}

impl QueryPayload {
    pub fn new(query: &str) -> Self {
        Self {
            query: query.to_string(),
        }
    }
}

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
    pub async fn query(&self, payload: QueryPayload) -> anyhow::Result<Vec<SearchResultData>> {
        let text_embedding = self.ctx.text_embedding()?.0;
        let multi_modal_embedding: TextEmbeddingModel = self.ctx.multi_modal_embedding()?.0.into();
        let vision_collection_name = self.vision_collection_name.as_str();
        let language_collection_name = self.language_collection_name.as_str();

        let clip_text_embedding = multi_modal_embedding
            .process_single(payload.query.clone())
            .await?;
        let text_model_embedding = text_embedding.process_single(payload.query.clone()).await?;

        let mut retrieval_results = HashMap::new();

        let text_search = async {
            let payload = SearchPointsBuilder::new(
                language_collection_name,
                text_model_embedding.clone(),
                RETRIEVAL_COUNT,
            )
            .with_payload(true)
            .score_threshold(0.5);

            self.qdrant.search_points(payload).await
        };

        let vision_search = async {
            let payload = SearchPointsBuilder::new(
                vision_collection_name,
                clip_text_embedding.clone(),
                RETRIEVAL_COUNT,
            )
            .with_payload(true)
            .score_threshold(0.2);

            self.qdrant.search_points(payload).await
        };

        let (text_result, vision_result) = tokio::join!(text_search, vision_search);

        let text_response = text_result?;
        group_results_by_asset(&text_response.result, &mut retrieval_results);

        let vision_response = vision_result?;
        let vision_points: Vec<_> = vision_response
            .result
            .into_iter()
            .map(|mut v| {
                v.score += 0.5;
                v
            })
            .collect();
        group_results_by_asset(&vision_points, &mut retrieval_results);

        Ok(reorder_final_results(&mut retrieval_results)?)
    }

    /// 实现基于文本特征的基础召回
    pub async fn retrieve(
        &self,
        payload: QueryPayload,
    ) -> anyhow::Result<Vec<RetrievalResultData>> {
        let text_embedding = self.ctx.text_embedding()?.0;
        let language_collection_name = self.language_collection_name.as_str();
        let text_model_embedding = text_embedding.process_single(payload.query.clone()).await?;

        let payload = SearchPointsBuilder::new(
            language_collection_name,
            text_model_embedding.clone(),
            5,
        )
        .with_payload(true);

        let response = self.qdrant.search_points(payload).await?;

        Ok(response
            .result
            .into_iter()
            .filter_map(|v| {
                if let Ok(payload) = serde_json::from_value::<SearchPayload>(json!(v.payload)) {
                    Some(RetrievalResultData {
                        score: v.score,
                        file_identifier: payload.file_identifier,
                        task_type: payload.task_type,
                        metadata: payload.metadata,
                    })
                } else {
                    None
                }
            })
            .collect())
    }
}
