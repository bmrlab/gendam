use crate::db::entity::frame::{AudioFrameEntity, ImageFrameEntity};
use crate::db::entity::page::PageEntity;
use crate::db::entity::relation::RelationEntity;
use crate::db::entity::{
    AudioEntity, DocumentEntity, ImageEntity, PayloadEntity, TextEntity, VideoEntity, WebPageEntity,
};
use crate::db::model::id::{ID, TB};
use crate::db::DB;
use anyhow::bail;
use async_recursion::async_recursion;

impl DB {
    /// - 传入顶层元素，比如 video，则会递归删除子元素
    ///
    /// - 如果传入底层元素，比如 text，则不会向上删除，只会删除 text 本身
    ///
    /// - 如果有 contain 关系将被删除
    ///
    /// - 如果有 with 关系也会被删除，对应的 payload 也会被同时删除
    ///
    /// 删除流程
    /// 1. 查询是否有 with 关系
    ///     - 如果有，删除 with 关系
    ///     - 删除 payload
    /// 2. 删除元素
    ///
    /// tips：级联删除暂不支持
    /// https://github.com/surrealdb/surrealdb/issues/1374
    pub async fn delete(&self, id: &ID) -> anyhow::Result<()> {
        match id.tb() {
            TB::Text => self.delete_text(id).await,
            TB::Image => self.delete_image(id).await,
            TB::ImageFrame => self.delete_image_frame(id).await,
            TB::AudioFrame => self.delete_audio_frame(id).await,
            TB::Audio => self.delete_audio(id).await,
            TB::Video => self.delete_video(id).await,
            TB::Page => self.delete_page(id).await,
            TB::Web => self.delete_web(id).await,
            TB::Document => self.delete_document(id).await,
            TB::Payload => self.delete_payload(id).await,
            _ => bail!("not implemented"),
        }
    }

    /// - `with` 关系会被删除
    ///     - 如果有 `with` 关系，对应的 payload 也会被删除，payload 关系是一对一的
    ///
    /// - `contain` 关系会被删除，并且子元素也将被递归删除（如果子元素有 `contain` 也会被删除）
    async fn delete_text(&self, id: &ID) -> anyhow::Result<()> {
        self.delete_with_relation_and_payload_by_id(id).await?;
        self.client
            .delete::<Option<TextEntity>>((id.table_name(), id.id()))
            .await?;
        Ok(())
    }

    async fn delete_image(&self, id: &ID) -> anyhow::Result<()> {
        self.delete_with_relation_and_payload_by_id(id).await?;
        self.client
            .delete::<Option<ImageEntity>>((id.table_name(), id.id()))
            .await?;
        Ok(())
    }

    /// 1. 查找 image frame 的 contain 关系
    /// 2. 删除 contain 关系
    /// 3. 删除子 record 记录
    /// 4. 删除 image frame
    #[async_recursion]
    async fn delete_image_frame(&self, id: &ID) -> anyhow::Result<()> {
        self.delete_contain_relation_and_subrecord_by_id(id).await?;
        self.client
            .delete::<Option<ImageFrameEntity>>((id.table_name(), id.id()))
            .await?;
        Ok(())
    }

    #[async_recursion]
    async fn delete_audio_frame(&self, id: &ID) -> anyhow::Result<()> {
        self.delete_contain_relation_and_subrecord_by_id(id).await?;
        self.client
            .delete::<Option<AudioFrameEntity>>((id.table_name(), id.id()))
            .await?;
        Ok(())
    }

    async fn delete_audio(&self, id: &ID) -> anyhow::Result<()> {
        self.delete_with_relation_and_payload_by_id(id).await?;
        self.client
            .delete::<Option<AudioEntity>>((id.table_name(), id.id()))
            .await?;
        Ok(())
    }

    async fn delete_video(&self, id: &ID) -> anyhow::Result<()> {
        self.delete_with_relation_and_payload_by_id(id).await?;
        self.client
            .delete::<Option<VideoEntity>>((id.table_name(), id.id()))
            .await?;
        Ok(())
    }

    async fn delete_page(&self, id: &ID) -> anyhow::Result<()> {
        self.delete_with_relation_and_payload_by_id(id).await?;
        self.client
            .delete::<Option<PageEntity>>((id.table_name(), id.id()))
            .await?;
        Ok(())
    }

    async fn delete_web(&self, id: &ID) -> anyhow::Result<()> {
        self.delete_with_relation_and_payload_by_id(id).await?;
        self.client
            .delete::<Option<WebPageEntity>>((id.table_name(), id.id()))
            .await?;
        Ok(())
    }

    async fn delete_document(&self, id: &ID) -> anyhow::Result<()> {
        self.delete_with_relation_and_payload_by_id(id).await?;
        self.client
            .delete::<Option<DocumentEntity>>((id.table_name(), id.id()))
            .await?;
        Ok(())
    }

    /// id: payload id
    async fn delete_payload(&self, id: &ID) -> anyhow::Result<()> {
        self.client
            .delete::<Option<PayloadEntity>>((id.table_name(), id.id()))
            .await?;
        Ok(())
    }

    async fn batch_delete_payload(&self, ids: Vec<&ID>) -> anyhow::Result<()> {
        for id in ids {
            if let Err(e) = self.delete_payload(id).await {
                tracing::error!("failed to delete payload {id:?}: {e}");
            }
        }
        Ok(())
    }
}

/// 删除 relation
impl DB {
    /// - id：只要 id，不要 table_name
    ///
    /// - 因为没有把 contain 当作一张表，TB 中没有 contain
    async fn delete_contain_relation(&self, id: &str) -> anyhow::Result<()> {
        self.client
            .delete::<Option<RelationEntity>>(("contain", id))
            .await?;
        Ok(())
    }

    /// - id：只要 id，不要 table_name
    ///
    /// - 因为没有把 with 当作一张表，TB 中没有 with
    async fn delete_with_relation(&self, id: &str) -> anyhow::Result<()> {
        self.client
            .delete::<Option<RelationEntity>>(("with", id))
            .await?;
        Ok(())
    }

    async fn batch_delete_with_relation(&self, ids: Vec<String>) -> anyhow::Result<()> {
        for id in ids {
            if let Err(e) = self.delete_with_relation(id.as_str()).await {
                tracing::error!("failed to delete with relation {id:?}: {e}");
            }
        }
        Ok(())
    }

    /// - 删除 with relation 和 payload
    ///
    /// id: video、audio、document、web、text、image
    async fn delete_with_relation_and_payload_by_id(&self, id: &ID) -> anyhow::Result<()> {
        let relations = self.select_with_relation_by_in(id).await?;
        let with_ids = relations.iter().map(|x| x.id()).collect::<Vec<String>>();
        let payload_ids = relations
            .iter()
            .map(|x| ID::from(x.out_id().as_str()))
            .collect::<Vec<ID>>();

        self.batch_delete_with_relation(with_ids).await?;
        self.batch_delete_payload(payload_ids.iter().collect())
            .await?;
        Ok(())
    }

    /// - 删除 contain relation 和 subrecord
    ///
    /// id: video、audio、document、web、page、image_frame、audio_frame
    async fn delete_contain_relation_and_subrecord_by_id(&self, id: &ID) -> anyhow::Result<()> {
        let relations = self.select_contains_relation_by_in(id).await?;
        for relation in relations {
            self.delete_contain_relation_and_subrecord_by_relation(&relation)
                .await?;
        }
        Ok(())
    }

    /// 根据 relation 来删除 contain 和 subrecord
    ///
    /// subrecord 不可能是 video、audio、document、web 这些顶层元素
    async fn delete_contain_relation_and_subrecord_by_relation(
        &self,
        relation: &RelationEntity,
    ) -> anyhow::Result<()> {
        let subrecord_id = ID::from(relation.out_id().as_str());

        // 删除 contain relation
        self.delete_contain_relation(relation.id_without_table().as_str())
            .await?;

        // 处理 subrecord
        match subrecord_id.tb() {
            TB::Text => {
                self.delete_text(&subrecord_id).await?;
            }
            TB::Image => {
                self.delete_image(&subrecord_id).await?;
            }
            TB::ImageFrame => {
                self.delete_image_frame(&subrecord_id).await?;
            }
            TB::AudioFrame => {
                self.delete_audio_frame(&subrecord_id).await?;
            }
            TB::Page => {
                self.delete_page(&subrecord_id).await?;
            }
            _ => bail!("can not reach here"),
        }
        Ok(())
    }
}

#[allow(unused)]
mod test {
    use crate::db::model::id::ID;
    use crate::db::shared::test::setup;
    use crate::db::DB;

    async fn local_db() -> DB {
        setup(Some(r#"/Users/zingerbee/Library/Application Support/ai.gendam.desktop/libraries/185e94cf-5e4b-4723-94a4-238068edd50a/surreal"#.as_ref())).await
    }

    #[tokio::test]
    async fn test_delete_text() {
        let db = local_db().await;
        let res = db.delete_text(&ID::from("text:qd1osplk9jqb2xafatd5")).await;
        assert!(res.is_ok())
    }

    #[tokio::test]
    async fn test_delete_contain_relation() {
        let db = local_db().await;
        let res = db.delete_contain_relation("x7bb2jx7oqw6aj7fbpm2").await;
        assert!(res.is_ok())
    }
}
