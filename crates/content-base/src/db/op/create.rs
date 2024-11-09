use crate::{
    check_db_error_from_resp,
    db::{
        model::{
            audio::AudioModel, document::DocumentModel, id::ID, image::ImageModel, page::PageModel,
            payload::PayloadModel, text::TextModel, video::VideoModel, web::WebPageModel,
        },
        DB,
    },
};

/// insert api
impl DB {
    pub async fn insert_image(
        &self,
        image_model: ImageModel,
        file_identifier: Option<String>,
    ) -> anyhow::Result<ID> {
        let record = ImageModel::create_only(&self.client, &image_model).await?;
        if let Some(file_identifier) = file_identifier {
            PayloadModel::create_for_model(&self.client, &record, &file_identifier.into()).await?;
        }
        Ok(ID::from(record))
    }

    pub async fn insert_text(&self, text_model: TextModel) -> anyhow::Result<ID> {
        let record = TextModel::create_only(&self.client, &text_model).await?;
        Ok(ID::from(record))
    }

    pub async fn insert_audio(
        &self,
        audio_model: AudioModel,
        file_identifier: String,
    ) -> anyhow::Result<ID> {
        let record = AudioModel::create_only(&self.client, &audio_model).await?;
        PayloadModel::create_for_model(&self.client, &record, &file_identifier.into()).await?;
        Ok(ID::from(record))
    }

    pub async fn insert_video(
        &self,
        video_model: VideoModel,
        file_identifier: String,
    ) -> anyhow::Result<ID> {
        let record = VideoModel::create_only(&self.client, &video_model).await?;
        PayloadModel::create_for_model(&self.client, &record, &file_identifier.into()).await?;
        Ok(ID::from(record))
    }

    pub async fn insert_web_page(
        &self,
        web_page_model: WebPageModel,
        file_identifier: String,
    ) -> anyhow::Result<ID> {
        let record = WebPageModel::create_only(&self.client, &web_page_model).await?;
        PayloadModel::create_for_model(&self.client, &record, &file_identifier.into()).await?;
        Ok(ID::from(record))
    }

    pub async fn insert_document(
        &self,
        document_model: DocumentModel,
        file_identifier: String,
    ) -> anyhow::Result<ID> {
        let record = DocumentModel::create_only(&self.client, &document_model).await?;
        PayloadModel::create_for_model(&self.client, &record, &file_identifier.into()).await?;
        Ok(ID::from(record))
    }
}

/// use for test
impl DB {
    #[allow(dead_code)]
    pub(crate) async fn upsert(&self, id: &ID, set_clause: &str) -> anyhow::Result<()> {
        let mut resp = self
            .client
            .query(format!("UPSERT {} SET {};", id.id_with_table(), set_clause))
            .await?;
        check_db_error_from_resp!(resp).map_err(|errors_map| {
            tracing::error!("upsert errors: {:?}", errors_map);
            anyhow::anyhow!("Failed to upsert, errors: {:?}", errors_map)
        })?;
        Ok(())
    }

    #[allow(dead_code)]
    pub(crate) async fn insert_page(&self, page_model: PageModel) -> anyhow::Result<ID> {
        let record = PageModel::create_only(&self.client, &page_model).await?;
        Ok(ID::from(record))
    }
}

#[allow(unused_imports, dead_code)]
mod test {
    use crate::check_db_error_from_resp;
    use crate::db::entity::TextEntity;
    use crate::db::model::id::{ID, TB};
    use crate::db::model::{image::ImageModel, text::TextModel};
    use crate::db::shared::test::{
        fake_audio_model, fake_document, fake_file_identifier, fake_image_model, fake_page_model,
        fake_video_model, fake_web_page_model, gen_vector, setup,
    };
    use crate::db::DB;
    use itertools::Itertools;
    use std::process::id;
    use test_log::test;

    // 让 test 串行执行
    static TEST_LOCK: std::sync::OnceLock<tokio::sync::Mutex<()>> = std::sync::OnceLock::new();
    async fn get_test_lock() -> &'static tokio::sync::Mutex<()> {
        TEST_LOCK.get_or_init(|| tokio::sync::Mutex::new(()))
    }

    #[test(tokio::test)]
    async fn test_insert_text() {
        let _guard = get_test_lock().await.lock().await;
        let id = setup(None)
            .await
            .insert_text(TextModel {
                id: None,
                data: "data".to_string(),
                vector: gen_vector(1024),
                en_data: "en_data".to_string(),
                en_vector: gen_vector(1024),
            })
            .await
            .unwrap();
        println!("{:?}", id);
        assert_eq!(id.tb(), &TB::Text);
    }

    #[test(tokio::test)]
    async fn test_insert_image() {
        let _guard = get_test_lock().await.lock().await;
        let db = setup(None).await;
        let _ = db
            .insert_image(fake_image_model(), Some(fake_file_identifier()))
            .await;
    }

    #[test(tokio::test)]
    async fn test_insert_audio() {
        let _guard = get_test_lock().await.lock().await;
        let db = setup(None).await;
        let id = db
            .insert_audio(fake_audio_model(), fake_file_identifier())
            .await
            .unwrap();
        assert_eq!(id.tb(), &TB::Audio);
    }

    #[test(tokio::test)]
    async fn test_insert_video() {
        let _guard = get_test_lock().await.lock().await;
        let db = setup(None).await;
        let id = db
            .insert_video(fake_video_model(), fake_file_identifier())
            .await
            .unwrap();
        assert_eq!(id.tb(), &TB::Video);
    }

    #[test(tokio::test)]
    async fn test_insert_page() {
        let _guard = get_test_lock().await.lock().await;
        let db = setup(None).await;
        let id = db.insert_page(fake_page_model()).await.unwrap();
        assert_eq!(id.tb(), &TB::Page);
    }

    #[test(tokio::test)]
    async fn test_insert_web_page() {
        let _guard = get_test_lock().await.lock().await;
        let db = setup(None).await;
        let id = db
            .insert_web_page(fake_web_page_model(), fake_file_identifier())
            .await
            .unwrap();
        assert_eq!(id.tb(), &TB::Web);
    }

    #[test(tokio::test)]
    async fn test_insert_document() {
        let _guard = get_test_lock().await.lock().await;
        let db = setup(None).await;
        let id = db
            .insert_document(fake_document(), fake_file_identifier())
            .await
            .unwrap();
        assert_eq!(id.tb(), &TB::Document);
    }

    #[test(tokio::test)]
    async fn test_upsert() {
        let _guard = get_test_lock().await.lock().await;
        let db = setup(None).await;
        db.upsert(
            &ID::from("text:11232131"),
            format!(
                "data = 't-1', vector = [{}], en_data = 't-1', en_vector = [{}]",
                gen_vector(1024)
                    .into_iter()
                    .map(|v| v.to_string())
                    .join(","),
                gen_vector(1024)
                    .into_iter()
                    .map(|v| v.to_string())
                    .join(",")
            )
            .as_str(),
        )
        .await
        .unwrap();

        let mut resp = db
            .client
            .query(format!("SELECT * FROM {};", "text:11232131"))
            .await
            .unwrap();
        check_db_error_from_resp!(resp)
            .map_err(|errors_map| anyhow::anyhow!("select text error: {:?}", errors_map))
            .unwrap();
        let result = resp.take::<Vec<TextEntity>>(0).unwrap();
        assert_eq!(result.len(), 1);
    }
}
