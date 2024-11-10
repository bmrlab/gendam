use super::model::SelectResultModel;
use crate::query::model::{FullTextSearchResult, HitResult};

#[macro_export]
macro_rules! check_db_error_from_resp {
    ($resp:ident) => {{
        let errors_map = $resp.take_errors();
        if !errors_map.is_empty() {
            Err(errors_map)
        } else {
            Ok(())
        }
    }};
}

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

// TODO: 没有考虑 en_data 的情况
// 没必要这样，直接在 hitresult 上加一个 highlight 就行了
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
