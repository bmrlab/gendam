use crate::{
    check_db_error_from_resp,
    db::{
        entity::{
            AudioEntity, DocumentEntity, ImageEntity, PayloadEntity, TextEntity, VideoEntity,
            WebPageEntity,
        },
        DB,
    },
};
use futures::{stream, StreamExt};

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
