use super::{model::SearchModel, ContentQueryPayload};
use crate::{
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
    #[tracing::instrument(err(Debug), skip_all, fields(query=%payload.query))]
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
            text_embedding: text_model_embedding,
            vision_embedding: clip_text_embedding,
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
}
