use ai::TextEmbeddingModel;
use regex::Regex;

use super::{model::SearchModel, QueryPayload};
use crate::db::model::{PayloadModel, SelectResultModel};
use crate::query::payload::audio::AudioSearchMetadata;
use crate::query::payload::image::ImageSearchMetadata;
use crate::query::payload::raw_text::RawTextSearchMetadata;
use crate::query::payload::web_page::WebPageSearchMetadata;
use crate::query::payload::{SearchMetadata, SearchResultData};
use crate::{
    concat_arrays,
    constant::STOP_WORDS,
    query::model::{TextSearchModel, TextToken},
    utils::deduplicate,
    ContentBase,
};

impl ContentBase {
    /// 构造内部查询模型 SearchModel
    /// 1. 分词
    /// 2. query 文字 -> 文本向量 + 图片向量
    pub async fn query_payload_to_model(
        &self,
        payload: QueryPayload,
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

    pub async fn expand_select_result(
        &self,
        select_result: &SelectResultModel,
        score: f32,
        payload: &PayloadModel,
    ) -> anyhow::Result<Vec<SearchResultData>> {
        let file_identifier = payload.file_identifier();
        let metadata = match select_result {
            SelectResultModel::Image(_) => vec![SearchMetadata::Image(ImageSearchMetadata {})],
            SelectResultModel::Audio(audio) => audio
                .audio_frame
                .iter()
                .map(|frame| {
                    SearchMetadata::Audio(AudioSearchMetadata {
                        start_timestamp: frame.start_timestamp as i64,
                        end_timestamp: frame.end_timestamp as i64,
                    })
                })
                .collect(),
            SelectResultModel::Video(video) => {
                let audio_metadata = video
                    .audio_frame
                    .iter()
                    .map(|frame| {
                        SearchMetadata::Audio(AudioSearchMetadata {
                            start_timestamp: frame.start_timestamp as i64,
                            end_timestamp: frame.end_timestamp as i64,
                        })
                    })
                    .collect::<Vec<SearchMetadata>>();
                let image_metadata = video
                    .image_frame
                    .iter()
                    .map(|_| SearchMetadata::Image(ImageSearchMetadata {}))
                    .collect::<Vec<SearchMetadata>>();

                concat_arrays!(audio_metadata, image_metadata).into_vec()
            }
            SelectResultModel::WebPage(web_page) => web_page
                .page
                .iter()
                .map(|page| {
                    SearchMetadata::WebPage(WebPageSearchMetadata {
                        start_index: page.start_index as usize,
                        end_index: page.end_index as usize,
                    })
                })
                .collect(),
            SelectResultModel::Document(document) => document
                .page
                .iter()
                .map(|page| {
                    SearchMetadata::RawText(RawTextSearchMetadata {
                        start_index: page.start_index as usize,
                        end_index: page.end_index as usize,
                    })
                })
                .collect(),
            _ => {
                vec![]
            }
        };
        Ok(metadata
            .into_iter()
            .map(|metadata| SearchResultData {
                file_identifier: file_identifier.clone(),
                score,
                metadata,
            })
            .collect())
    }
}
