use anyhow::bail;
use futures::future::join_all;
use std::convert::Into;
use tracing::{debug, error};

use super::{constant::MAX_FULLTEXT_TOKEN, entity::vector::VectorSearchEntity, DB};
use crate::db::entity::full_text::FullTextWithHighlightSearchEntity;
use crate::db::entity::relation::RelationEntity;
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
    pub async fn full_text_search(
        &self,
        data: Vec<String>,
        with_highlight: bool,
    ) -> anyhow::Result<Vec<FullTextSearchResult>> {
        Ok(if with_highlight {
            self.full_text_search_with_highlight(data).await?
        } else {
            self._full_text_search(data).await?
        })
    }

    /// 🔍 full text search
    /// 对每个分词进行全文搜索
    /// 分词之间使用 OR 连接
    /// 缺点是高亮结果是分散的
    async fn _full_text_search(
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

        Ok(join_all(futures)
            .await
            .into_iter()
            .collect::<anyhow::Result<Vec<_>>>()?
            .into_iter()
            .flatten()
            .collect())
    }

    /// 全文搜索并高亮
    /// 将整个搜索结果丢进去，然后返回高亮结果
    /// 分词之间的结果是 AND 连接
    /// 缺点是无法直接确定命中了哪个分词
    ///    - 可以通过正则 <b></b> 来确定关键词
    async fn full_text_search_with_highlight(
        &self,
        data: Vec<String>,
    ) -> anyhow::Result<Vec<FullTextSearchResult>> {
        if data.is_empty() {
            return Ok(vec![]);
        }
        let data = data.join(" ");

        let futures = FULL_TEXT_SEARCH_TABLE.iter().map(|table| {
            let sql = format!(
                "SELECT id, search::score(0) as score, search::highlight('<b>', '</b>', 0) AS highlight FROM {} WHERE {} LIMIT {};",
                table.table_name(),
                format!("{} @0@ '{}'", table.column_name(), data),
                SELEC_LIMIT
            );
            debug!(
                "full-text search with highlight on table {}: {sql}",
                table.table_name()
            );

            async move {
                let mut resp = self.client.query(&sql).await?;
                check_db_error_from_resp!(resp).map_err(|errors_map| {
                    error!("full_text_search_with_highlight errors: {errors_map:?}");
                    anyhow::anyhow!("Failed to full_text_search_with_highlight")
                })?;
                let text: Vec<FullTextWithHighlightSearchEntity> = resp.take(0)?;
                Ok::<_, anyhow::Error>(
                    text.into_iter()
                        .map(Into::into)
                        .collect::<Vec<FullTextSearchResult>>(),
                )
            }
        });

        Ok(join_all(futures)
            .await
            .into_iter()
            .collect::<anyhow::Result<Vec<_>>>()?
            .into_iter()
            .flatten()
            .collect())
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
pub struct BacktrackResult {
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
    pub async fn backtrace_by_ids(&self, ids: Vec<ID>) -> anyhow::Result<Vec<BacktrackResult>> {
        let backtrace = stream::iter(ids)
            .then(|id| async move {
                let mut res: Vec<BacktrackResult> = vec![];
                let has_relation = self.has_contains_relation(&id).await?;
                if has_relation {
                    let backtrace_relation =
                        self.backtrace_relation(vec![id.id_with_table()]).await?;

                    debug!(
                        "backtrace_relation: {:?}",
                        backtrace_relation
                            .iter()
                            .map(|r| (r.hit_id.clone(), r.result.clone()))
                            .collect::<Vec<(Vec<ID>, RelationEntity)>>()
                    );

                    for br in backtrace_relation {
                        let entity = self.select_entity_by_relation(&br.result).await?;
                        for select_entity in entity {
                            res.push(BacktrackResult {
                                origin_id: id.clone(),
                                hit_id: br.hit_id.clone(),
                                result: select_entity.into(),
                            });
                        }
                    }
                } else {
                    // 没有 contain 关系的情况
                    let data = match id.tb() {
                        TB::Text => self
                            .select_text(vec![id.id_with_table()])
                            .await?
                            .into_iter()
                            .map(SelectResultEntity::Text)
                            .collect::<Vec<SelectResultEntity>>()
                            .pop(),
                        TB::Image => self
                            .select_image(vec![id.id_with_table()])
                            .await?
                            .into_iter()
                            .map(SelectResultEntity::Image)
                            .collect::<Vec<SelectResultEntity>>()
                            .pop(),
                        _ => {
                            error!("should not be here: {:?}", id);
                            None
                        }
                    };
                    if let Some(entity) = data {
                        res.push(BacktrackResult {
                            origin_id: id.clone(),
                            hit_id: vec![id.clone()],
                            result: entity.into(),
                        });
                    }
                }
                Ok::<Vec<BacktrackResult>, anyhow::Error>(res)
            })
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .filter_map(Result::ok)
            .flatten()
            .collect::<Vec<BacktrackResult>>();

        Ok(backtrace)
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
    use crate::db::model::id::ID;
    use crate::db::model::video::VideoModel;
    use crate::db::shared::test::{
        fake_upsert_text_clause, fake_video_model, fake_video_payload, gen_vector, setup,
    };
    use crate::query::payload::video::VideoSearchMetadata;
    use crate::query::payload::{SearchMetadata, SearchPayload};
    use content_base_task::video::VideoTaskType;
    use content_base_task::ContentTaskType;
    use itertools::Itertools;
    use std::process::id;
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
        let single_text_id = ID::from("text:11232131");
        db.upsert(&single_text_id, fake_upsert_text_clause().as_str())
            .await
            .unwrap();

        let video_id = db
            .insert_video(fake_video_model(), fake_video_payload())
            .await
            .unwrap();

        let mut video: VideoModel = db
            .select_video(vec![video_id.id_with_table()])
            .await
            .unwrap()
            .pop()
            .unwrap()
            .into();

        println!("video: {video:?}");

        if video.audio_frame.is_empty() {
            println!("audio_frame is empty skip");
            return;
        }

        let mut audio_frame = video.audio_frame.pop().unwrap();
        println!("audio_frame: {:?}", audio_frame.id);

        let text = audio_frame.data.pop().unwrap();
        println!("text: {:?}", text);

        let res = db
            .backtrace_by_ids(vec![text.id.unwrap(), single_text_id.clone()])
            .await
            .unwrap();
        println!("res: {:?}", res[0]);
        println!("single_res: {:?}", res[1]);
        assert_eq!(res.len(), 2);
        assert!(res[0].hit_id.len() > 0);
        assert_eq!(res[1].hit_id.len(), 1);
        assert_eq!(res[1].hit_id[0], single_text_id);
    }

    #[test(tokio::test)]
    async fn test_full_text_search_with_highlight() {
        let db = setup().await;
        let res = db
            .full_text_search_with_highlight(vec!["LVL小河板".to_string()])
            .await
            .unwrap();
        println!("res: {res:#?}");
    }
}
