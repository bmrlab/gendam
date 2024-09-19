use crate::db::{
    entity::{
        relation::RelationEntity, AudioEntity, DocumentEntity, ImageEntity, SelectResultEntity,
        TextEntity, VideoEntity, WebPageEntity,
    },
    model::id::{ID, TB},
    DB,
};
use crate::utils::deduplicate;
use futures::{stream, StreamExt};
use itertools::Itertools;
use tracing::error;

macro_rules! select_some_macro {
    ($fetch:expr, $client:expr, $ids:expr, $return_type:ty) => {{
        let mut result = vec![];

        stream::iter($ids)
            .then(|id| async move {
                let mut resp = $client
                    .query(format!("SELECT * FROM {} {};", id.as_ref(), $fetch))
                    .await?;
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

/// audio -> audio_frame -> text
/// 当 TB 在 MIDDLE_LAYER_LIST 中时，需要继续向上查询一层，才是最终的结果
const MIDDLE_LAYER_LIST: [TB; 3] = [TB::AudioFrame, TB::ImageFrame, TB::Page];

// 数据查询
impl DB {
    /// ids: 只包含 text 和 image 表的 ID
    /// ids 是去重的
    /// 查询出的结果顺序是和 ids 一致的
    pub async fn select_by_id(&self, ids: Vec<ID>) -> anyhow::Result<Vec<SelectResultEntity>> {
        let mut backtrack = vec![];
        stream::iter(ids)
            .then(|id| async move {
                let mut res = vec![];
                let relation_by_out = self
                    .select_relation_by_out(vec![id.id_with_table()])
                    .await?;
                let relation = relation_by_out
                    .iter()
                    .map(|r| r.in_id())
                    .collect::<Vec<_>>();
                if !relation.is_empty() {
                    let relation = deduplicate(relation)
                        .into_iter()
                        .filter_map(|id| relation_by_out.iter().find(|r| r.in_id() == id))
                        .collect::<Vec<&RelationEntity>>();

                    stream::iter(relation)
                        .then(|r| async move {
                            let res = match r.in_table() {
                                TB::Audio => self
                                    .select_audio(vec![r.in_id()])
                                    .await?
                                    .into_iter()
                                    .map(SelectResultEntity::Audio)
                                    .collect::<Vec<SelectResultEntity>>(),
                                TB::Video => self
                                    .select_video(vec![r.in_id()])
                                    .await?
                                    .into_iter()
                                    .map(SelectResultEntity::Video)
                                    .collect::<Vec<SelectResultEntity>>(),
                                TB::Web => self
                                    .select_web_page(vec![r.in_id()])
                                    .await?
                                    .into_iter()
                                    .map(SelectResultEntity::WebPage)
                                    .collect::<Vec<SelectResultEntity>>(),
                                TB::Document => self
                                    .select_document(vec![r.in_id()])
                                    .await?
                                    .into_iter()
                                    .map(SelectResultEntity::Document)
                                    .collect::<Vec<SelectResultEntity>>(),
                                _ => {
                                    error!("select_by_id inner error: {:?}", r);
                                    vec![]
                                }
                            };
                            Ok::<_, anyhow::Error>(res)
                        })
                        .collect::<Vec<_>>()
                        .await
                        .into_iter()
                        .for_each(|select_entity| match select_entity {
                            Ok(s) => {
                                res.push(s);
                            }
                            _ => {}
                        });
                } else {
                    // 没有 contain 关系的情况
                    match id.tb() {
                        TB::Text => {
                            let text = self.select_text(vec![id.id_with_table()]).await?;
                            res.push(
                                text.into_iter()
                                    .map(SelectResultEntity::Text)
                                    .collect::<Vec<SelectResultEntity>>(),
                            );
                        }
                        TB::Image => {
                            let image = self.select_image(vec![id.id_with_table()]).await?;
                            res.push(
                                image
                                    .into_iter()
                                    .map(SelectResultEntity::Image)
                                    .collect::<Vec<SelectResultEntity>>(),
                            );
                        }
                        _ => {
                            error!("should not be here: {:?}", id);
                        }
                    }
                }
                Ok::<Vec<SelectResultEntity>, anyhow::Error>(res.into_iter().flatten().collect())
            })
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .for_each(|res| match res {
                Ok(res) => {
                    backtrack.push(res);
                }
                _ => {
                    error!("select_by_id out error: {:?}", res);
                }
            });

        Ok(backtrack.into_iter().flatten().collect())
    }

    /// 检查 id 是否存在 contain 关系
    /// 如果是多层 contain，则返回最顶层的 id
    /// page、audio_frame、image_frame 都是中间层，还需要向上查询
    async fn select_relation_by_out(
        &self,
        ids: Vec<impl AsRef<str>>,
    ) -> anyhow::Result<Vec<RelationEntity>> {
        let futures = stream::iter(ids)
            .then(|id| async move {
                Ok::<_, anyhow::Error>(
                    self.client
                        .query(format!(
                            "SELECT * from contains where out = {};",
                            id.as_ref()
                        ))
                        .await?
                        .take::<Vec<RelationEntity>>(0)?,
                )
            })
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .filter_map(Result::ok)
            .flatten()
            .map(|r| async move {
                match r.in_table() {
                    // 可以确定在 MIDDLE_LAYER_LIST 表中的还有一层 contain 关系
                    tb if MIDDLE_LAYER_LIST.contains(&tb) => Ok::<_, anyhow::Error>(
                        self.client
                            .query(format!("SELECT * from contains where out = {};", r.in_id()))
                            .await?
                            .take::<Vec<RelationEntity>>(0)?,
                    ),
                    _ => Ok::<_, anyhow::Error>(vec![r]),
                }
            })
            .collect::<Vec<_>>();

        let futures_result: Vec<Vec<RelationEntity>> = stream::iter(futures)
            .buffered(1)
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .filter(Result::is_ok)
            .try_collect()?;

        Ok(futures_result
            .into_iter()
            .flatten()
            .collect::<Vec<RelationEntity>>())
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
}

#[allow(unused_imports)]
mod test {
    use crate::{
        db::{
            model::{id::TB, ImageModel, TextModel},
            shared::test::{gen_vector, setup},
        },
        query::payload::{SearchMetadata, SearchPayload},
    };
    use content_base_task::{
        web_page::{transform::WebPageTransformTask, WebPageTaskType},
        ContentTaskType,
    };
    use itertools::Itertools;
    use test_log::test;

    #[test(tokio::test)]
    async fn test_select_relation_by_out() {
        let db = setup().await;
        // Document data needs to be inserted in advance
        // can insert data by running the test in `create/mod`
        let document_res = db
            .select_relation_by_out(vec!["image:5it65bxgm0u603livkv8"])
            .await
            .unwrap();
        // the desired result is document
        println!("document_res: {:?}", document_res);
        // Video data needs to be inserted in advance
        let video_res = db
            .select_relation_by_out(vec!["text:vu3lb2verv2h36hti5im"])
            .await
            .unwrap();
        // the desired result is video
        println!("video_res: {:?}", video_res);
        // Text data needs to be inserted in advance
        // no relation data
        let text_res = db
            .select_relation_by_out(vec!["text:qtx3nucfeo7rzm3mun5b"])
            .await
            .unwrap();
        // the desired result is empty
        println!("text_res: {:?}", text_res);
        // Combine data needs to be inserted in advance
        // audio and no relation data
        let combine_res = db
            .select_relation_by_out(vec![
                "text:hkot8rlbc8ogoiwoxnms",
                "text:qtx3nucfeo7rzm3mun5b",
            ])
            .await
            .unwrap();
        // the desired result only is audio
        println!("combine_res: {:?}", combine_res);
    }

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
    async fn test_select_by_id() {
        let db = setup().await;
        let res = db
            .select_by_id(vec![
                "text:0k611fzdax6vdqexqv82".into(),
                "text:1xv13ncm0i0h3ykhv1t2".into(),
                "text:2uftzfxknwiu0iasroxw".into(),
                "text:7r2g1vj5ennxtbi0hp5a".into(),
                "text:aw2cyxkvukk6gvy20x4r".into(),
            ])
            .await
            .unwrap();
        println!("res: {:?}", res.into_iter().map(|r| r.id()).collect_vec());
    }
}
