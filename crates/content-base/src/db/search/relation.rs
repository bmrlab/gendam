use crate::db::entity::{PayloadEntity, SelectResultEntity};
use crate::db::model::id::ID;
use crate::db::{entity::relation::RelationEntity, model::id::TB, DB};
use futures::{stream, StreamExt};
use itertools::Itertools;
use tracing::error;

/// audio -> audio_frame -> text
/// 当 TB 在 MIDDLE_LAYER_LIST 中时，需要继续向上查询一层，才是最终的结果
const MIDDLE_LAYER_LIST: [TB; 3] = [TB::AudioFrame, TB::ImageFrame, TB::Page];
const HAS_PAYLOAD_LIST: [TB; 6] = [
    TB::Text,
    TB::Image,
    TB::Audio,
    TB::Video,
    TB::Web,
    TB::Document,
];

// 数据查询
impl DB {
    /// 检查 id 是否存在 contain 关系
    /// 如果是多层 contain，则返回最顶层的 id
    /// page、audio_frame、image_frame 都是中间层，还需要向上查询
    pub async fn select_relation_by_out(
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

    pub async fn select_payload_by_ids(&self, id: Vec<ID>) -> anyhow::Result<Vec<PayloadEntity>> {
        stream::iter(id)
            .then(|id| self.select_payload_by_id(id))
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .collect()
    }

    /// `with` payload only has one
    pub async fn select_payload_by_id(&self, id: ID) -> anyhow::Result<PayloadEntity> {
        if HAS_PAYLOAD_LIST.contains(&id.tb()) {
            let relation = self.select_with_relation(&id).await?.pop().ok_or_else(|| {
                anyhow::anyhow!("no relation data under {} table", id.id_with_table())
            })?;

            self.select_payload(vec![relation.in_id()])
                .await?
                .pop()
                .ok_or_else(|| {
                    anyhow::anyhow!("no payload data under {} table", id.id_with_table())
                })
        } else {
            Err(anyhow::anyhow!(
                "no payload under {} table",
                id.table_name()
            ))
        }
    }

    /// only in HAS_PAYLOAD_LIST table has payload
    async fn select_with_relation(&self, id: &ID) -> anyhow::Result<Vec<RelationEntity>> {
        self.client
            .query(format!(
                "SELECT * from with where in = {};",
                id.id_with_table()
            ))
            .await?
            .take::<Vec<RelationEntity>>(0)
            .map_err(Into::into)
    }

    pub async fn select_entity_by_relation(
        &self,
        relation: &RelationEntity,
    ) -> anyhow::Result<Vec<SelectResultEntity>> {
        let in_id = relation.in_id();
        Ok(match relation.in_table() {
            TB::Text => self
                .select_text(vec![in_id])
                .await?
                .into_iter()
                .map(SelectResultEntity::Text)
                .collect::<Vec<SelectResultEntity>>(),
            TB::Image => self
                .select_image(vec![in_id])
                .await?
                .into_iter()
                .map(SelectResultEntity::Image)
                .collect::<Vec<SelectResultEntity>>(),
            TB::Audio => self
                .select_audio(vec![in_id])
                .await?
                .into_iter()
                .map(SelectResultEntity::Audio)
                .collect::<Vec<SelectResultEntity>>(),
            TB::Video => self
                .select_video(vec![in_id])
                .await?
                .into_iter()
                .map(SelectResultEntity::Video)
                .collect::<Vec<SelectResultEntity>>(),
            TB::Web => self
                .select_web_page(vec![in_id])
                .await?
                .into_iter()
                .map(SelectResultEntity::WebPage)
                .collect::<Vec<SelectResultEntity>>(),
            TB::Document => self
                .select_document(vec![in_id])
                .await?
                .into_iter()
                .map(SelectResultEntity::Document)
                .collect::<Vec<SelectResultEntity>>(),
            TB::Payload => self
                .select_payload(vec![in_id])
                .await?
                .into_iter()
                .map(SelectResultEntity::Payload)
                .collect::<Vec<SelectResultEntity>>(),
            _ => {
                error!("select entity by relation {relation:?} error: no implementation");
                vec![]
            }
        })
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
    async fn test_select_payload_by_id() {
        let db = setup().await;
        // let res = db
        //     .select_payload_by_id("text:vu3lb2verv2h36hti5im".into())
        //     .await;
        // println!("res: {:?}", res);
        // assert!(res.is_err());

        // let res = db
        //     .select_payload_by_id("document:6dr6glzpf7ixefh7vjks".into())
        //     .await;
        // println!("res: {:?}", res);
        // assert!(res.is_ok());
        
        
        // TODO: debug
        let res = db
            .select_payload_by_id("video:6tqtjseeuln7l9xus7t2".into())
            .await;
        println!("res: {:?}", res);
        assert!(res.is_ok())
    }
}
