use crate::db::model::id::{ID, TB};
use crate::db::DB;
use anyhow::bail;

impl DB {
    /// 传入顶层元素，比如 video，则会递归删除子元素
    /// 如果传入底层元素，比如 text，则不会向上删除，只会删除 text 本身
    /// 如果有 contain 关系将被删除
    /// 如果有 with 关系也会被删除，对应的 payload 也会被同时删除
    pub async fn delete(&self, id: &ID) -> anyhow::Result<()> {
        match id.tb() {
            TB::Text => self.delete_text(id).await,
            TB::Image => self.delete_image(id).await,
            TB::Item => self.delete_item(id).await,
            TB::ImageFrame => self.delete_image_frame(id).await,
            TB::AudioFrame => self.delete_audio_frame(id).await,
            TB::Audio => self.delete_audio(id).await,
            TB::Video => self.delete_video(id).await,
            TB::Page => self.delete_page(id).await,
            TB::Web => self.delete_web(id).await,
            TB::Document => self.delete_document(id).await,
            TB::Payload => self.delete_payload(id).await,
        }
    }

    /// 以下的接口，只会删除元素本身，不会删除 relate 关系
    async fn delete_text(&self, id: &ID) -> anyhow::Result<()> {
        // delete text
        Ok(())
    }

    async fn delete_image(&self, id: &ID) -> anyhow::Result<()> {
        // delete image
        Ok(())
    }

    async fn delete_item(&self, _: &ID) -> anyhow::Result<()> {
        // delete item
        bail!("delete item not implemented");
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
        // delete payload
        Ok(())
    }
}
