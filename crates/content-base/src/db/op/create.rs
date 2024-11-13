use crate::{
    check_db_error_from_resp,
    db::{
        model::{
            audio::{AudioFrameModel, AudioModel},
            document::DocumentModel,
            id::ID,
            image::ImageModel,
            page::PageModel,
            payload::PayloadModel,
            text::TextModel,
            video::{ImageFrameModel, VideoModel},
            web::WebPageModel,
            ModelCreate,
        },
        DB,
    },
};

/// insert api
impl DB {
    pub async fn insert_image(
        &self,
        file_identifier: Option<String>,
        image: ImageModel,
    ) -> anyhow::Result<ID> {
        let record = ImageModel::create_only(&self.client, &image).await?;
        if let Some(file_identifier) = file_identifier {
            PayloadModel::create_for_model(&self.client, &record, &file_identifier.into()).await?;
        }
        Ok(ID::from(record))
    }

    pub async fn insert_text(
        &self,
        _file_identifier: Option<String>,
        text: TextModel,
    ) -> anyhow::Result<ID> {
        let record = TextModel::create_only(&self.client, &text).await?;
        Ok(ID::from(record))
    }

    pub async fn insert_audio(
        &self,
        file_identifier: String,
        (audio_model, audio_frames): (AudioModel, Vec<(AudioFrameModel, Vec<TextModel>)>),
    ) -> anyhow::Result<ID> {
        let record = AudioModel::create_only(&self.client, &(audio_model, audio_frames)).await?;
        PayloadModel::create_for_model(&self.client, &record, &file_identifier.into()).await?;
        Ok(ID::from(record))
    }

    pub async fn insert_video(
        &self,
        file_identifier: String,
        (video, image_frames, audio_frames): (
            VideoModel,
            Vec<(ImageFrameModel, Vec<ImageModel>)>,
            Vec<(AudioFrameModel, Vec<TextModel>)>,
        ),
    ) -> anyhow::Result<ID> {
        let record =
            VideoModel::create_only(&self.client, &(video, image_frames, audio_frames)).await?;
        PayloadModel::create_for_model(&self.client, &record, &file_identifier.into()).await?;
        Ok(ID::from(record))
    }

    pub async fn insert_web_page(
        &self,
        file_identifier: String,
        (web_page, pages): (
            WebPageModel,
            Vec<(PageModel, Vec<TextModel>, Vec<ImageModel>)>,
        ),
    ) -> anyhow::Result<ID> {
        let record = WebPageModel::create_only(&self.client, &(web_page, pages)).await?;
        PayloadModel::create_for_model(&self.client, &record, &file_identifier.into()).await?;
        Ok(ID::from(record))
    }

    pub async fn insert_document(
        &self,
        file_identifier: String,
        (document, pages): (
            DocumentModel,
            Vec<(PageModel, Vec<TextModel>, Vec<ImageModel>)>,
        ),
    ) -> anyhow::Result<ID> {
        let record = DocumentModel::create_only(&self.client, &(document, pages)).await?;
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
    pub(crate) async fn insert_page(
        &self,
        (page, page_texts, page_images): (PageModel, Vec<TextModel>, Vec<ImageModel>),
    ) -> anyhow::Result<ID> {
        let record = PageModel::create_only(&self.client, &(page, page_texts, page_images)).await?;
        Ok(ID::from(record))
    }
}
