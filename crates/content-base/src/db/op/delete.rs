use crate::db::entity::relation::RelationEntity;
use crate::db::entity::{ImageEntity, PayloadEntity, TextEntity};
use crate::db::model::id::{ID, TB};
use crate::db::DB;
use anyhow::bail;

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

    /// - 不会删除 `with` 关系
    /// 
    /// - `contain` 关系会被删除，并且子元素也将被递归删除（如果子元素有 `contain` 也会被删除）
    async fn delete_text(&self, id: &ID) -> anyhow::Result<()> {
        self.client
            .delete::<Option<TextEntity>>((id.table_name(), id.id()))
            .await?;
        Ok(())
    }

    async fn delete_image(&self, id: &ID) -> anyhow::Result<()> {
        self.client
            .delete::<Option<ImageEntity>>((id.table_name(), id.id()))
            .await?;
        Ok(())
    }

    async fn delete_image_frame(&self, id: &ID) -> anyhow::Result<()> {
        // delete image frame
        Ok(())
    }

    async fn delete_audio_frame(&self, id: &ID) -> anyhow::Result<()> {
        // delete audio frame
        Ok(())
    }

    async fn delete_audio(&self, id: &ID) -> anyhow::Result<()> {
        // delete audio
        Ok(())
    }

    async fn delete_video(&self, id: &ID) -> anyhow::Result<()> {
        // delete video
        Ok(())
    }

    async fn delete_page(&self, id: &ID) -> anyhow::Result<()> {
        // delete page
        Ok(())
    }

    async fn delete_web(&self, id: &ID) -> anyhow::Result<()> {
        // delete web
        Ok(())
    }

    async fn delete_document(&self, id: &ID) -> anyhow::Result<()> {
        // delete document
        Ok(())
    }

    async fn delete_payload(&self, id: &ID) -> anyhow::Result<()> {
        self.client
            .delete::<Option<PayloadEntity>>((id.table_name(), id.id()))
            .await?;
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
