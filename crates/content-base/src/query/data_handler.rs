use super::{model::SearchModel, HitResult, QueryPayload};
use crate::db::model::SelectResultModel;
use crate::query::model::full_text::FullTextSearchResult;
use crate::query::payload::audio::AudioSearchMetadata;
use crate::query::payload::image::ImageSearchMetadata;
use crate::query::payload::video::VideoSearchMetadata;
use crate::query::payload::web_page::WebPageSearchMetadata;
use crate::query::payload::{SearchMetadata, SearchResultData};
use crate::{
    concat_arrays,
    constant::STOP_WORDS,
    query::model::{TextSearchModel, TextToken},
    utils::deduplicate,
    ContentBase,
};
use ai::TextEmbeddingModel;
use regex::Regex;

macro_rules! replace_data {
    ($id:expr, $replace_id:expr, $replace_data:expr, $target:expr) => {
        if $id.as_ref().map_or(false, |inner| inner == $replace_id) {
            *$target = $replace_data.to_string().into();
        }
    };
}

macro_rules! replace_in_frames {
    ($frames:expr, $replace_id:expr, $replace_data:expr, $data_field:ident) => {
        $frames.iter_mut().for_each(|frame| {
            frame.data.iter_mut().for_each(|f| {
                replace_data!(&mut f.id, $replace_id, $replace_data, &mut f.$data_field);
            });
        });
    };
}

macro_rules! replace_in_pages {
    ($pages:expr, $replace_id:expr, $replace_data:expr) => {
        $pages.iter_mut().for_each(|page| {
            page.text.iter_mut().for_each(|p| {
                replace_data!(&mut p.id, $replace_id, $replace_data, &mut p.data);
            });
            page.image.iter_mut().for_each(|p| {
                replace_data!(&mut p.id, $replace_id, $replace_data, &mut p.prompt);
            });
        });
    };
}

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

    pub fn expand_hit_result(
        &self,
        hit_result: HitResult,
    ) -> anyhow::Result<Vec<SearchResultData>> {
        let file_identifier = hit_result.payload.file_identifier();
        let metadata = match hit_result.result {
            SelectResultModel::Image(_) => vec![SearchMetadata::Image(ImageSearchMetadata {})],
            SelectResultModel::Audio(audio) => audio
                .audio_frame
                .iter()
                .filter_map(|frame| {
                    frame.id.as_ref().and_then(|frame_id| {
                        if hit_result.hit_id.contains(frame_id) {
                            Some(SearchMetadata::Audio(AudioSearchMetadata {
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
                                Some(SearchMetadata::Video(VideoSearchMetadata {
                                    start_timestamp: frame.start_timestamp as i64,
                                    end_timestamp: frame.end_timestamp as i64,
                                }))
                            } else {
                                None
                            }
                        })
                    })
                    .collect::<Vec<SearchMetadata>>();
                let image_metadata = video
                    .image_frame
                    .iter()
                    .filter_map(|frame| {
                        frame.id.as_ref().and_then(|frame_id| {
                            if hit_result.hit_id.contains(frame_id) {
                                Some(SearchMetadata::Video(VideoSearchMetadata {
                                    start_timestamp: frame.start_timestamp as i64,
                                    end_timestamp: frame.end_timestamp as i64,
                                }))
                            } else {
                                None
                            }
                        })
                    })
                    .collect::<Vec<SearchMetadata>>();

                concat_arrays!(audio_metadata, image_metadata).into_vec()
            }
            SelectResultModel::WebPage(web_page) => web_page
                .page
                .iter()
                .filter_map(|page| {
                    page.id.as_ref().and_then(|page_id| {
                        if hit_result.hit_id.contains(page_id) {
                            Some(SearchMetadata::WebPage(WebPageSearchMetadata {
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
                            Some(SearchMetadata::WebPage(WebPageSearchMetadata {
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
        };
        Ok(metadata
            .into_iter()
            .map(|metadata| SearchResultData {
                file_identifier: file_identifier.clone(),
                score: hit_result.score,
                metadata,
            })
            .collect())
    }

    // TODO: 没有考虑 en_data 的情况
    pub fn replace_with_highlight(
        full_text: Vec<FullTextSearchResult>,
        hit_results: Vec<HitResult>,
    ) -> Vec<HitResult> {
        hit_results
            .into_iter()
            .map(|mut h| {
                h.result = match h.result {
                    SelectResultModel::Text(mut text) => {
                        full_text.iter().for_each(|ft| {
                            replace_data!(&mut text.id, &ft.id, &ft.score[0].0, &mut text.data);
                        });
                        SelectResultModel::Text(text)
                    }
                    SelectResultModel::Image(mut image) => {
                        full_text.iter().for_each(|ft| {
                            replace_data!(&mut image.id, &ft.id, &ft.score[0].0, &mut image.prompt);
                        });
                        SelectResultModel::Image(image)
                    }
                    SelectResultModel::Audio(mut audio) => {
                        full_text.iter().for_each(|ft| {
                            replace_in_frames!(audio.audio_frame, &ft.id, &ft.score[0].0, data);
                        });
                        SelectResultModel::Audio(audio)
                    }
                    SelectResultModel::Video(mut video) => {
                        full_text.iter().for_each(|ft| {
                            replace_in_frames!(video.audio_frame, &ft.id, &ft.score[0].0, data);
                            replace_in_frames!(video.image_frame, &ft.id, &ft.score[0].0, prompt);
                        });
                        SelectResultModel::Video(video)
                    }
                    SelectResultModel::WebPage(mut web) => {
                        full_text.iter().for_each(|ft| {
                            replace_in_pages!(web.page, &ft.id, &ft.score[0].0);
                        });
                        SelectResultModel::WebPage(web)
                    }
                    SelectResultModel::Document(mut document) => {
                        full_text.iter().for_each(|ft| {
                            replace_in_pages!(document.page, &ft.id, &ft.score[0].0);
                        });
                        SelectResultModel::Document(document)
                    }
                    _ => h.result,
                };
                h
            })
            .collect::<Vec<HitResult>>()
    }
}
