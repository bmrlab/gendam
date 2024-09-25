use anyhow::bail;
use futures::future::join_all;
use tracing::{debug, error};

use super::{constant::MAX_FULLTEXT_TOKEN, entity::vector::VectorSearchEntity, DB};
use crate::db::entity::{
    AudioEntity, DocumentEntity, ImageEntity, PayloadEntity, SelectResultEntity, TextEntity,
    VideoEntity, WebPageEntity,
};
use crate::db::model::id::{ID, TB};
use crate::db::model::SelectResultModel;
use crate::query::model::vector::VectorSearchTable;
use crate::{
    check_db_error_from_resp,
    db::{constant::SELEC_LIMIT, entity::full_text::FullTextSearchEntity},
    query::model::{
        full_text::{FullTextSearchResult, FULL_TEXT_SEARCH_TABLE},
        vector::{VectorSearchResult, VECTOR_SEARCH_TABLE},
    },
};
use futures::{stream, StreamExt};

mod relation;

macro_rules! select_some_macro {
    ($fetch:expr, $client:expr, $ids:expr, $return_type:ty) => {{
        let mut result = vec![];

        stream::iter($ids)
            .then(|id| async move {
                let mut resp = $client
                    .query(format!("SELECT * FROM {} {};", id.as_ref(), $fetch))
                    .await?;
                check_db_error_from_resp!(resp).map_err(|errors_map| {
                    error!("select_some_macro errors: {errors_map:?}");
                    anyhow::anyhow!("Failed to select some")
                })?;
                let result = resp.take::<Vec<$return_type>>(0)?;
                Ok::<_, anyhow::Error>(result)
            })
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .for_each(|res| match res {
                Ok(image) => {
                    result.push(image);
                }
                Err(e) => {
                    error!("select error: {e:?}");
                }
            });
        Ok(result.into_iter().flatten().collect())
    }};
}

// search
impl DB {
    /// 🔍 full text search
    pub async fn full_text_search(
        &self,
        data: Vec<String>,
    ) -> anyhow::Result<Vec<FullTextSearchResult>> {
        if data.is_empty() {
            return Ok(vec![]);
        }
        let data = if data.len() <= MAX_FULLTEXT_TOKEN {
            &data[..]
        } else {
            &data[0..MAX_FULLTEXT_TOKEN]
        };

        let futures = FULL_TEXT_SEARCH_TABLE.iter().map(|table| {
            let param_sql = |data: (usize, &String)| -> (String, String) {
                (
                    format!("search::score({}) AS score_{}", data.0, data.0),
                    format!("{} @{}@ '{}'", table.column_name(), data.0, data.1),
                )
            };

            let (search_scores, where_clauses): (Vec<_>, Vec<_>) =
                data.iter().enumerate().map(param_sql).unzip();

            let sql = format!(
                "SELECT id, {} FROM {} WHERE {} LIMIT {};",
                search_scores.join(", "),
                table.table_name(),
                where_clauses.join(" OR "),
                SELEC_LIMIT
            );
            debug!(
                "full-text search sql on table {}: {sql}",
                table.table_name()
            );

            let data: Vec<String> = data.into_iter().map(|d| d.to_string()).collect();
            async move {
                let text: Vec<FullTextSearchEntity> = self.client.query(&sql).await?.take(0)?;
                Ok::<_, anyhow::Error>(
                    text.iter()
                        .map(|t| t.convert_to_result(&data))
                        .collect::<Vec<_>>(),
                )
            }
        });

        let res: Vec<FullTextSearchResult> = join_all(futures)
            .await
            .into_iter()
            .collect::<anyhow::Result<Vec<_>>>()?
            .into_iter()
            .flatten()
            .collect();
        Ok(res)
    }

    /// 🔍 vector search
    /// if not vision_vector, please input text_vector
    pub async fn vector_search(
        &self,
        text_vector: Vec<f32>,
        vision_vector: Vec<f32>,
        range: Option<&str>,
    ) -> anyhow::Result<Vec<VectorSearchResult>> {
        if text_vector.is_empty() || vision_vector.is_empty() {
            bail!("data is empty in vector search");
        }
        let range = range.unwrap_or_else(|| "<|10,40|>");
        let futures = VECTOR_SEARCH_TABLE.map(|v| {
            let data = match v {
                VectorSearchTable::Text => text_vector.clone(),
                VectorSearchTable::EnText => text_vector.clone(),
                VectorSearchTable::Image => vision_vector.clone(),
                VectorSearchTable::ImagePrompt => text_vector.clone(),
            };
            async move {
                let mut res = self
                    .client
                    .query(format!("SELECT id, vector::distance::knn() AS distance FROM {} WHERE {} {} $vector ORDER BY distance LIMIT {};", v.table_name(), v.column_name(), range, SELEC_LIMIT))
                    .bind(("vector", data))
                    .await?;
                let res: Vec<VectorSearchEntity> = res.take(0)?;
                Ok::<_, anyhow::Error>(res.iter().map(|d| d.into()).collect::<Vec<VectorSearchResult>>())
            }
        });

        let mut res: Vec<VectorSearchResult> = join_all(futures)
            .await
            .into_iter()
            .collect::<anyhow::Result<Vec<_>>>()?
            .into_iter()
            .flatten()
            .collect();
        res.sort_by(|a, b| {
            a.distance
                .partial_cmp(&b.distance)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        Ok(res)
    }
}

#[derive(Debug)]
pub struct BacktraceResult {
    /// 只包含 text 和 image 表的 ID
    pub origin_id: ID,
    /// 命中的 id
    /// 如果 origin_id 没有 relation，则是 origin_id
    /// 如果 origin_id 有 relation
    ///     - video 类型，则是 audio_frame、image_frame
    ///     - web 类型，则是 page
    ///     - document 类型，则是 page
    pub hit_id: Vec<ID>,
    pub result: SelectResultModel,
}

impl DB {
    /// ids: 只包含 text 和 image 表的 ID
    /// ids 是去重的
    /// 查询出的结果顺序是和 ids 一致的
    pub async fn backtrace_by_ids(&self, ids: Vec<ID>) -> anyhow::Result<Vec<BacktraceResult>> {
        let backtrack = stream::iter(ids)
            .then(|id| async move {
                let mut res: Vec<BacktraceResult> = vec![];
                let has_relation = self.has_contains_relation(&id).await?;
                if has_relation {
                    let backtrack_relation =
                        self.backtrack_relation(vec![id.id_with_table()]).await?;

                    for br in backtrack_relation {
                        let entity = self.select_entity_by_relation(&br.result).await?;
                        for select_entity in entity {
                            res.push(BacktraceResult {
                                origin_id: id.clone(),
                                hit_id: br.hit_id.clone(),
                                result: select_entity.into(),
                            });
                        }
                    }
                } else {
                    // 没有 contain 关系的情况
                    match id.tb() {
                        TB::Text => {
                            let text = self
                                .select_text(vec![id.id_with_table()])
                                .await?
                                .into_iter()
                                .map(SelectResultEntity::Text)
                                .collect::<Vec<SelectResultEntity>>()
                                .pop();
                            if let Some(text) = text {
                                res.push(BacktraceResult {
                                    origin_id: id.clone(),
                                    hit_id: vec![id.clone()],
                                    result: text.into(),
                                });
                            }
                        }
                        TB::Image => {
                            let image = self
                                .select_image(vec![id.id_with_table()])
                                .await?
                                .into_iter()
                                .map(SelectResultEntity::Image)
                                .collect::<Vec<SelectResultEntity>>()
                                .pop();
                            if let Some(image) = image {
                                res.push(BacktraceResult {
                                    origin_id: id.clone(),
                                    hit_id: vec![id.clone()],
                                    result: image.into(),
                                });
                            }
                        }
                        _ => {
                            error!("should not be here: {:?}", id);
                        }
                    }
                }
                Ok::<Vec<BacktraceResult>, anyhow::Error>(res)
            })
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .filter_map(Result::ok)
            .flatten()
            .collect::<Vec<BacktraceResult>>();

        Ok(backtrack)
    }

    async fn select_text(&self, ids: Vec<impl AsRef<str>>) -> anyhow::Result<Vec<TextEntity>> {
        select_some_macro!("", self.client, ids, TextEntity)
    }

    async fn select_image(&self, ids: Vec<impl AsRef<str>>) -> anyhow::Result<Vec<ImageEntity>> {
        select_some_macro!("", self.client, ids, ImageEntity)
    }

    async fn select_audio(&self, ids: Vec<impl AsRef<str>>) -> anyhow::Result<Vec<AudioEntity>> {
        select_some_macro!("FETCH frame, frame.data", self.client, ids, AudioEntity)
    }

    async fn select_video(&self, ids: Vec<impl AsRef<str>>) -> anyhow::Result<Vec<VideoEntity>> {
        select_some_macro!(
            "FETCH image_frame, audio_frame, image_frame.data, audio_frame.data",
            self.client,
            ids,
            VideoEntity
        )
    }

    async fn select_web_page(
        &self,
        ids: Vec<impl AsRef<str>>,
    ) -> anyhow::Result<Vec<WebPageEntity>> {
        select_some_macro!(
            "FETCH page, page.text, page.image",
            self.client,
            ids,
            WebPageEntity
        )
    }

    async fn select_document(
        &self,
        ids: Vec<impl AsRef<str>>,
    ) -> anyhow::Result<Vec<DocumentEntity>> {
        select_some_macro!(
            "FETCH page, page.text, page.image",
            self.client,
            ids,
            DocumentEntity
        )
    }

    async fn select_payload(
        &self,
        ids: Vec<impl AsRef<str>>,
    ) -> anyhow::Result<Vec<PayloadEntity>> {
        select_some_macro!("", self.client, ids, PayloadEntity)
    }
}

#[allow(unused_imports)]
mod test {
    use crate::db::model::video::VideoModel;
    use crate::db::shared::test::{fake_video_model, fake_video_payload, setup};
    use crate::query::payload::video::VideoSearchMetadata;
    use crate::query::payload::{SearchMetadata, SearchPayload};
    use content_base_task::video::VideoTaskType;
    use content_base_task::ContentTaskType;
    use itertools::Itertools;
    use test_log::test;

    #[test(tokio::test)]
    async fn test_select_text() {
        let db = setup().await;
        let text_res = db
            .select_text(vec!["text:7dd12x11yvt5fgamdjb0"])
            .await
            .unwrap();
        println!("text_res: {:?}", text_res);
    }

    #[test(tokio::test)]
    async fn test_select_image() {
        let db = setup().await;
        let image_res = db
            .select_image(vec!["image:flzkn6ncniglqttxnrsm"])
            .await
            .unwrap();
        println!("image_res: {:?}", image_res);
    }

    #[test(tokio::test)]
    async fn test_select_audio() {
        let db = setup().await;
        let audio_res = db
            .select_audio(vec!["audio:gkzq6db9jwr34l3j0gmz"])
            .await
            .unwrap();
        println!("audio_res: {:?}", audio_res);
    }

    #[test(tokio::test)]
    async fn test_select_video() {
        let db = setup().await;
        let video_res = db
            .select_video(vec!["video:u456grwuvl6w74zgqemc"])
            .await
            .unwrap();
        println!("video_res: {:?}", video_res);
    }

    #[test(tokio::test)]
    async fn test_select_web_page() {
        let db = setup().await;
        let web_page_res = db
            .select_web_page(vec!["web:nobc02c8ffyol3kqbsln"])
            .await
            .unwrap();
        println!("web_page_res: {:?}", web_page_res);
    }

    #[test(tokio::test)]
    async fn test_select_document() {
        let db = setup().await;
        let document_res = db
            .select_document(vec!["document:6dr6glzpf7ixefh7vjks"])
            .await
            .unwrap();
        println!("document_res: {:?}", document_res);
    }

    #[test(tokio::test)]
    async fn test_backtrace_by_ids() {
        let db = setup().await;
        let video_id = db
            .insert_video(fake_video_model(), fake_video_payload())
            .await
            .unwrap();

        let video: VideoModel = db
            .select_video(vec![video_id.id_with_table()])
            .await
            .unwrap()
            .pop()
            .unwrap()
            .into();
        
        let text_id = video.audio_frame[0].data[0].data.clone();

        let res = db
            .backtrace_by_ids(vec![
                "text:0k611fzdax6vdqexqv82".into(),
                "text:1xv13ncm0i0h3ykhv1t2".into(),
                "text:2uftzfxknwiu0iasroxw".into(),
                "text:7r2g1vj5ennxtbi0hp5a".into(),
                "text:aw2cyxkvukk6gvy20x4r".into(),
            ])
            .await
            .unwrap();
        println!(
            "res: {:?}",
            res.into_iter()
                .map(|r| (r.origin_id, r.hit_id, r.result))
                .collect_vec()
        );
    }
}
