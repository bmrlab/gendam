use crate::{
    check_db_error_from_resp, collect_ordered_async_results,
    db::{
        entity::{
            AudioEntity, DocumentEntity, ImageEntity, PayloadEntity, SelectResultEntity,
            TextEntity, VideoEntity, WebPageEntity,
        },
        model::{
            id::{ID, TB},
            SelectResultModel,
        },
        rank::Rank,
        utils::replace_with_highlight,
        DB,
    },
    query::model::{HitResult, SearchModel, SearchType},
    utils::extract_highlighted_content,
};
use futures::{stream, StreamExt};
use itertools::Itertools;
use std::convert::Into;

mod full_text_search;
mod relation;
mod vector_search;

macro_rules! select_some_macro {
    ($fetch:expr, $client:expr, $ids:expr, $return_type:ty) => {{
        let mut result = vec![];

        stream::iter($ids)
            .then(|id| async move {
                let mut resp = $client
                    .query(format!("SELECT * FROM {} {};", id.as_ref(), $fetch))
                    .await?;
                check_db_error_from_resp!(resp).map_err(|errors_map| {
                    tracing::error!("select_some_macro errors: {errors_map:?}");
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
                    tracing::error!("select error: {e:?}");
                }
            });
        Ok(result.into_iter().flatten().collect())
    }};
}

impl DB {
    pub async fn search(
        &self,
        data: SearchModel,
        with_highlight: bool,
        max_count: usize,
    ) -> anyhow::Result<Vec<HitResult>> {
        match data {
            SearchModel::Text(text) => {
                tracing::debug!("search tokens: {:?}", text.tokens.0);

                let mut full_text_results =
                    self.full_text_search(text.tokens.0, with_highlight).await?;
                tracing::debug!("{} found in full text search", full_text_results.len());

                let hit_words = full_text_results
                    .iter()
                    .map(|x| extract_highlighted_content(&x.score[0].0))
                    .flatten()
                    .collect::<Vec<String>>();
                tracing::debug!("hit words {hit_words:?}");

                let mut vector_results = self
                    .vector_search(text.text_vector, text.vision_vector)
                    .await?;
                tracing::debug!("{} found in vector search", vector_results.len());

                let rank_result = Rank::rank(
                    (&mut full_text_results, &mut vector_results),
                    false,
                    Some(max_count),
                )?;
                tracing::debug!("{} results after rank", rank_result.len());

                let backtrace_results = {
                    // 从 text 和 image 表回溯到关联的实体的结果，视频、音频、文档、网页、等
                    let search_ids: Vec<ID> =
                        rank_result.iter().map(|x| x.id.clone()).unique().collect();
                    self.backtrace_by_ids(search_ids).await?
                };
                tracing::debug!("{} backtrace results", backtrace_results.len());

                let mut hit_results = {
                    let futures = backtrace_results
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
                                .select_payload_by_id(bt.result.id().expect("id not found"))
                                .await?;
                            Ok::<_, anyhow::Error>((bt, score, search_type, payload).into())
                        })
                        .collect::<Vec<_>>();
                    collect_ordered_async_results!(futures, Vec<HitResult>)
                };

                if with_highlight {
                    hit_results = replace_with_highlight(full_text_results, hit_results);
                }

                tracing::debug!("{} final hit results", hit_results.len());
                Ok(hit_results)
            }
            _ => unimplemented!(),
        }
    }
}

#[derive(Debug)]
pub struct BacktrackResult {
    /// 只包含 text 和 image 表的 ID
    pub origin_id: ID,
    /// 命中的 id
    /// - 如果 origin_id 没有 relation，则是 origin_id
    /// - 如果 origin_id 有 relation
    ///     - video 类型，则是 audio_frame、image_frame
    ///     - web 类型，则是 page
    ///     - document 类型，则是 page
    pub hit_id: Vec<ID>,
    pub result: SelectResultModel,
}

impl DB {
    /// 从 text 和 image 表中根据 id 回溯关联的实体
    /// 查询出的结果顺序是和 ids（已去重）一致的
    ///
    /// 支持的情况:
    /// 1. ids 包含了 text 和 image 表中的一条记录，没有关联关系
    ///     - 直接返回对应记录
    /// 2. ids 包含了一条关联到 video、web_page、document 的 text 或 image 记录
    ///     - video: 返回包含了 audio_frame 和 image_frame 的 video 数据
    ///     - web_page: 返回包含了 text 和 image 列表的 web_page 数据
    ///     - document: 返回包含了 text 和 image 列表的 document 数据
    ///
    /// # 参数
    /// * `ids` - 只包含 text 和 image 表的 ID 列表（已去重）
    ///
    /// # 返回
    /// * `Vec<BacktrackResult>` - 按传入的 ids 顺序返回回溯结果列表
    pub async fn backtrace_by_ids(&self, ids: Vec<ID>) -> anyhow::Result<Vec<BacktrackResult>> {
        let backtrace = stream::iter(ids)
            .then(|id| async move {
                let mut res: Vec<BacktrackResult> = vec![];
                let has_relation = self.has_contains_relation(&id).await?;
                if has_relation {
                    let backtrace_relation =
                        self.backtrace_relation(vec![id.id_with_table()]).await?;

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
                            tracing::error!("should not be here: {:?}", id);
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

    pub(crate) async fn select_text(
        &self,
        ids: Vec<impl AsRef<str>>,
    ) -> anyhow::Result<Vec<TextEntity>> {
        select_some_macro!("", self.client, ids, TextEntity)
    }

    pub(crate) async fn select_image(
        &self,
        ids: Vec<impl AsRef<str>>,
    ) -> anyhow::Result<Vec<ImageEntity>> {
        select_some_macro!("", self.client, ids, ImageEntity)
    }

    pub(crate) async fn select_audio(
        &self,
        ids: Vec<impl AsRef<str>>,
    ) -> anyhow::Result<Vec<AudioEntity>> {
        select_some_macro!("FETCH frame, frame.data", self.client, ids, AudioEntity)
    }

    pub(crate) async fn select_video(
        &self,
        ids: Vec<impl AsRef<str>>,
    ) -> anyhow::Result<Vec<VideoEntity>> {
        select_some_macro!(
            "FETCH image_frame, audio_frame, image_frame.data, audio_frame.data",
            self.client,
            ids,
            VideoEntity
        )
    }

    pub(crate) async fn select_web_page(
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

    pub(crate) async fn select_document(
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

    pub(crate) async fn select_payload(
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
        fake_file_identifier, fake_upsert_text_clause, fake_video_model, gen_vector, setup,
    };
    use itertools::Itertools;
    use std::process::id;
    use test_log::test;

    #[test(tokio::test)]
    async fn test_select_text() {
        let db = setup(None).await;
        let text_res = db
            .select_text(vec!["text:7dd12x11yvt5fgamdjb0"])
            .await
            .unwrap();
        println!("text_res: {:?}", text_res);
    }

    #[test(tokio::test)]
    async fn test_select_image() {
        let db = setup(None).await;
        let image_res = db
            .select_image(vec!["image:flzkn6ncniglqttxnrsm"])
            .await
            .unwrap();
        println!("image_res: {:?}", image_res);
    }

    #[test(tokio::test)]
    async fn test_select_audio() {
        let db = setup(None).await;
        let audio_res = db
            .select_audio(vec!["audio:gkzq6db9jwr34l3j0gmz"])
            .await
            .unwrap();
        println!("audio_res: {:?}", audio_res);
    }

    #[test(tokio::test)]
    async fn test_select_video() {
        let db = setup(None).await;
        let video_res = db
            .select_video(vec!["video:u456grwuvl6w74zgqemc"])
            .await
            .unwrap();
        println!("video_res: {:?}", video_res);
    }

    #[test(tokio::test)]
    async fn test_select_web_page() {
        let db = setup(None).await;
        let web_page_res = db
            .select_web_page(vec!["web:nobc02c8ffyol3kqbsln"])
            .await
            .unwrap();
        println!("web_page_res: {:?}", web_page_res);
    }

    #[test(tokio::test)]
    async fn test_select_document() {
        let db = setup(None).await;
        let document_res = db
            .select_document(vec!["document:6dr6glzpf7ixefh7vjks"])
            .await
            .unwrap();
        println!("document_res: {:?}", document_res);
    }

    #[test(tokio::test)]
    async fn test_backtrace_by_ids() {
        let db = setup(None).await;
        let single_text_id = ID::from("text:11232131");
        db.upsert(&single_text_id, fake_upsert_text_clause().as_str())
            .await
            .unwrap();

        let video_id = db
            .insert_video(fake_video_model(), fake_file_identifier())
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
        let db = setup(None).await;
        let res = db
            .full_text_search_with_highlight(vec!["LVL小河板".to_string()])
            .await
            .unwrap();
        println!("res: {res:#?}");
    }
}
