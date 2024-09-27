use crate::db::model::id::ID;
use crate::db::search::BacktrackResult;
use crate::query::model::hit_result::HitResult;
use crate::query::model::SearchType;
use crate::query::rank::Rank;
use crate::{collect_ordered_async_results, ContentBase};
use itertools::Itertools;
use model::SearchModel;
use payload::{RetrievalResultData, SearchPayload, SearchResultData};
use qdrant_client::qdrant::SearchPointsBuilder;
use search::{group_results_by_asset, reorder_final_results};
use serde_json::json;
use tracing::debug;

mod data_handler;
pub mod model;
pub mod payload;
mod rank;
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
    // pub async fn query(&self, payload: QueryPayload) -> anyhow::Result<Vec<SearchResultData>> {
    //     let text_embedding = self.ctx.text_embedding()?.0;
    //     let multi_modal_embedding: TextEmbeddingModel = self.ctx.multi_modal_embedding()?.0.into();
    //     let vision_collection_name = self.vision_collection_name.as_str();
    //     let language_collection_name = self.language_collection_name.as_str();

    //     let clip_text_embedding = multi_modal_embedding
    //         .process_single(payload.query.clone())
    //         .await?;
    //     let text_model_embedding = text_embedding.process_single(payload.query.clone()).await?;

    //     let mut retrieval_results = HashMap::new();

    //     let text_search = async {
    //         let payload = SearchPointsBuilder::new(
    //             language_collection_name,
    //             text_model_embedding.clone(),
    //             RETRIEVAL_COUNT,
    //         )
    //         .with_payload(true)
    //         .score_threshold(0.5);

    //         self.qdrant.search_points(payload).await
    //     };

    //     let vision_search = async {
    //         let payload = SearchPointsBuilder::new(
    //             vision_collection_name,
    //             clip_text_embedding.clone(),
    //             RETRIEVAL_COUNT,
    //         )
    //         .with_payload(true)
    //         .score_threshold(0.2);

    //         self.qdrant.search_points(payload).await
    //     };

    //     let (text_result, vision_result) = tokio::join!(text_search, vision_search);

    //     let text_response = text_result?;
    //     group_results_by_asset(&text_response.result, &mut retrieval_results);

    //     let vision_response = vision_result?;
    //     let vision_points: Vec<_> = vision_response
    //         .result
    //         .into_iter()
    //         .map(|mut v| {
    //             v.score += 0.5;
    //             v
    //         })
    //         .collect();
    //     group_results_by_asset(&vision_points, &mut retrieval_results);

    //     Ok(reorder_final_results(&mut retrieval_results)?)
    // }

    /// 首先将入参转换为内部查询参数
    /// - 文本搜索流程
    ///     1. 获取全文搜索和向量搜索的结果（全文搜索和向量搜索只会搜索文本和图片）
    ///     2. 将上述结果进行 rank
    ///     3. 对上述 rank 的结果进行向上回溯
    ///     4. 填充 payload 信息
    pub async fn query(&self, payload: QueryPayload) -> anyhow::Result<Vec<SearchResultData>> {
        let with_highlight = true;
        // 目前 QueryPayload 只是文本
        match self.query_payload_to_model(payload).await? {
            SearchModel::Text(text) => {
                debug!("search tokens: {:?}", text.tokens.0);

                let full_text_result = self
                    .db
                    .try_read()?
                    .full_text_search(text.tokens.0, with_highlight)
                    .await?;

                debug!("full text result: {full_text_result:?}");
                let vector_result = self
                    .db
                    .try_read()?
                    .vector_search(text.text_vector, text.vision_vector, None)
                    .await?;

                let rank_result = Rank::rank(
                    (full_text_result.clone(), vector_result),
                    Some(true),
                    Some(10),
                )?;
                debug!("rank result: {rank_result:?}");
                let search_ids: Vec<ID> =
                    rank_result.iter().map(|x| x.id.clone()).unique().collect();
                debug!("search ids: {search_ids:?}");

                let select_by_id_result = self.db.try_read()?.backtrace_by_ids(search_ids).await?;
                debug!("select by id result: {select_by_id_result:?}");
                let hit_result_futures = select_by_id_result
                    .into_iter()
                    .filter_map(|backtrace| {
                        rank_result
                            .iter()
                            .find(|r| r.id.eq(&backtrace.origin_id))
                            .map(|r| (backtrace, r.score, r.search_type.clone()))
                    })
                    .collect::<Vec<(BacktrackResult, f32, SearchType)>>()
                    .into_iter()
                    .map(|(bt, score, search_type)| async move {
                        let payload = self
                            .db
                            .try_read()?
                            .select_payload_by_id(bt.result.id().expect("id not found"))
                            .await?;
                        Ok::<_, anyhow::Error>((bt, score, search_type, payload).into())
                    })
                    .collect::<Vec<_>>();

                let mut hit_result =
                    collect_ordered_async_results!(hit_result_futures, Vec<HitResult>);

                if with_highlight {
                    hit_result = Self::replace_with_highlight(full_text_result, hit_result);
                }
                debug!("hit result: {:#?}", hit_result);

                Ok(hit_result
                    .into_iter()
                    .filter_map(|hit| self.expand_hit_result(hit).ok())
                    .flatten()
                    .collect::<Vec<SearchResultData>>())
            }
            SearchModel::Image(_) => Ok(vec![]),
        }
    }

    /// 实现基于文本特征的基础召回
    pub async fn retrieve(
        &self,
        payload: QueryPayload,
    ) -> anyhow::Result<Vec<RetrievalResultData>> {
        let text_embedding = self.ctx.text_embedding()?.0;
        let language_collection_name = self.language_collection_name.as_str();
        let text_model_embedding = text_embedding.process_single(payload.query.clone()).await?;

        let payload =
            SearchPointsBuilder::new(language_collection_name, text_model_embedding.clone(), 5)
                .with_payload(true);

        let response = self.qdrant.search_points(payload).await?;

        Ok(response
            .result
            .into_iter()
            .filter_map(|v| {
                if let Ok(payload) = serde_json::from_value::<ContentIndexPayload>(json!(v.payload))
                {
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
