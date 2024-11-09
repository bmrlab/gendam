use super::{model::SearchModel, ContentQueryPayload, HitResult};
use crate::db::model::SelectResultModel;
use crate::query::payload::{
    audio::{AudioIndexMetadata, AudioSliceType},
    image::ImageIndexMetadata,
    raw_text::{RawTextChunkType, RawTextIndexMetadata},
    video::{VideoIndexMetadata, VideoSliceType},
    web_page::{WebPageChunkType, WebPageIndexMetadata},
    ContentIndexMetadata,
};
use crate::{
    concat_arrays,
    constant::STOP_WORDS,
    query::model::{TextSearchModel, TextToken},
    utils::deduplicate,
    ContentBase,
};
use ai::TextEmbeddingModel;
use regex::Regex;

impl ContentBase {
    /// 构造内部查询模型 SearchModel
    /// 1. 分词
    /// 2. query 文字 -> 文本向量 + 图片向量
    pub async fn query_payload_to_model(
        &self,
        payload: &ContentQueryPayload,
    ) -> anyhow::Result<SearchModel> {
        let multi_modal_embedding: TextEmbeddingModel = self.ctx.multi_modal_embedding()?.0.into();
        let clip_text_embedding = multi_modal_embedding
            .process_single(payload.query.clone())
            .await?;
        let text_model_embedding = self
            .ctx
            .text_embedding()?
            .0
            .process_single(payload.query.clone())
            .await?;
        Ok(SearchModel::Text(TextSearchModel {
            data: payload.query.clone(),
            tokens: TextToken(self.tokenizer(&payload.query).await?),
            text_vector: text_model_embedding,
            vision_vector: clip_text_embedding,
        }))
    }

    pub async fn tokenizer(&self, data: &str) -> anyhow::Result<Vec<String>> {
        let data = data.to_lowercase();
        // 匹配 STOP_WORDS 中的所有单词
        let pattern = format!(r"\b(?:{})\b", STOP_WORDS.join("|"));
        let re_stop_words = Regex::new(&pattern)?;

        // 匹配所有标点符号
        let punctuation_pattern = Regex::new(r"[^\w\s]|[！-～]")?;

        // 移除匹配的停用词
        let cleaned_data = re_stop_words.replace_all(data.as_str(), "");

        // 移除所有标点符号
        let final_result = punctuation_pattern.replace_all(&cleaned_data, "");

        let tokens: Vec<String> = final_result
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();

        Ok(deduplicate(tokens))
    }

    /// 搜索结果的命中数据展开为片段的 index metadata，针对每个命中的ID提取对应的时间戳、索引范围等信息
    pub fn expand_hit_result_to_index_metadata(
        &self,
        hit_result: &HitResult,
    ) -> Vec<ContentIndexMetadata> {
        match hit_result.result.clone() {
            SelectResultModel::Image(_) => {
                vec![ContentIndexMetadata::Image(ImageIndexMetadata { data: 0 })]
            }
            SelectResultModel::Audio(ref audio) => audio
                .audio_frame
                .iter()
                .filter_map(|frame| {
                    frame.id.as_ref().and_then(|frame_id| {
                        if hit_result.hit_id.contains(frame_id) {
                            Some(ContentIndexMetadata::Audio(AudioIndexMetadata {
                                slice_type: AudioSliceType::Transcript,
                                start_timestamp: frame.start_timestamp as i64,
                                end_timestamp: frame.end_timestamp as i64,
                            }))
                        } else {
                            None
                        }
                    })
                })
                .collect(),
            SelectResultModel::Video(video) => {
                let audio_metadata = video
                    .audio_frame
                    .iter()
                    .filter_map(|frame| {
                        frame.id.as_ref().and_then(|frame_id| {
                            if hit_result.hit_id.contains(frame_id) {
                                Some(ContentIndexMetadata::Video(VideoIndexMetadata {
                                    slice_type: VideoSliceType::Audio,
                                    start_timestamp: frame.start_timestamp as i64,
                                    end_timestamp: frame.end_timestamp as i64,
                                }))
                            } else {
                                None
                            }
                        })
                    })
                    .collect::<Vec<ContentIndexMetadata>>();
                let image_metadata = video
                    .image_frame
                    .iter()
                    .filter_map(|frame| {
                        frame.id.as_ref().and_then(|frame_id| {
                            if hit_result.hit_id.contains(frame_id) {
                                Some(ContentIndexMetadata::Video(VideoIndexMetadata {
                                    slice_type: VideoSliceType::Visual,
                                    start_timestamp: frame.start_timestamp as i64,
                                    end_timestamp: frame.end_timestamp as i64,
                                }))
                            } else {
                                None
                            }
                        })
                    })
                    .collect::<Vec<ContentIndexMetadata>>();

                concat_arrays!(audio_metadata, image_metadata).into_vec()
            }
            SelectResultModel::WebPage(web_page) => web_page
                .page
                .iter()
                .filter_map(|page| {
                    page.id.as_ref().and_then(|page_id| {
                        if hit_result.hit_id.contains(page_id) {
                            Some(ContentIndexMetadata::WebPage(WebPageIndexMetadata {
                                chunk_type: WebPageChunkType::Content,
                                start_index: page.start_index as usize,
                                end_index: page.end_index as usize,
                            }))
                        } else {
                            None
                        }
                    })
                })
                .collect(),
            SelectResultModel::Document(document) => document
                .page
                .iter()
                .filter_map(|page| {
                    page.id.as_ref().and_then(|page_id| {
                        if hit_result.hit_id.contains(page_id) {
                            Some(ContentIndexMetadata::RawText(RawTextIndexMetadata {
                                chunk_type: RawTextChunkType::Content,
                                start_index: page.start_index as usize,
                                end_index: page.end_index as usize,
                            }))
                        } else {
                            None
                        }
                    })
                })
                .collect(),
            _ => {
                vec![]
            }
        }
    }
}
