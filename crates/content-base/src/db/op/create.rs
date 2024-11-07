use anyhow::bail;
use surrealdb::sql::Thing;
use tracing::{debug, error};

use crate::db::model::audio::{AudioFrameModel, AudioModel};
use crate::db::model::document::DocumentModel;
use crate::db::model::id::ID;
use crate::db::model::payload::PayloadModel;
use crate::db::model::video::{ImageFrameModel, VideoModel};
use crate::db::model::web::WebPageModel;
use crate::db::model::{ImageModel, PageModel, TextModel};
use crate::db::DB;
use crate::{check_db_error_from_resp, collect_async_results, concat_arrays};

/// insert api
impl DB {
    pub async fn insert_image(
        &self,
        image_model: ImageModel,
        file_identifier: Option<String>,
    ) -> anyhow::Result<ID> {
        let mut resp = self
            .client
            .query(
                "
                (CREATE ONLY image CONTENT {
                    prompt: $prompt,
                    vector: $vector,
                    prompt_vector: $prompt_vector
                }).id",
            )
            .bind(image_model)
            .await?;

        check_db_error_from_resp!(resp).map_err(|errors_map| {
            error!("insert image errors: {:?}", errors_map);
            anyhow::anyhow!("Failed to insert image, errors: {:?}", errors_map)
        })?;

        let id: Option<ID> = resp.take::<Option<Thing>>(0)?.map(|x| x.into());

        match id {
            Some(id) => {
                if let Some(file_identifier) = file_identifier {
                    let payload_id = self.create_payload(file_identifier.into()).await?;
                    self.create_with_relation(&id, &payload_id).await?;
                }
                Ok(id)
            }
            None => {
                bail!("Failed to insert image");
            }
        }
    }

    pub async fn insert_audio(
        &self,
        audio: AudioModel,
        file_identifier: String,
    ) -> anyhow::Result<ID> {
        let ids = self
            .batch_insert_audio_frame(audio.audio_frame)
            .await?
            .into_iter()
            .map(|id| id.id_with_table())
            .collect::<Vec<String>>();
        if ids.is_empty() {
            error!("audio frame is empty");
            bail!("Failed to insert audio");
        }
        let create_audio_sql = format!(
            "(CREATE ONLY audio CONTENT {{ frame: [{}] }}).id",
            ids.join(", ")
        );
        let mut res = self.client.query(create_audio_sql).await?;
        match res.take::<Option<Thing>>(0)? {
            Some(id) => {
                let id: ID = id.into();
                self.create_contain_relation(
                    &id.id_with_table(),
                    ids.iter().map(|id| id.as_str()).collect(),
                )
                .await?;
                let payload_id = self.create_payload(file_identifier.into()).await?;
                self.create_with_relation(&id, &payload_id).await?;
                Ok(id)
            }
            None => Err(anyhow::anyhow!("Failed to insert audio")),
        }
    }

    pub async fn insert_video(
        &self,
        video: VideoModel,
        file_identifier: String,
    ) -> anyhow::Result<ID> {
        let image_frame_ids = self
            .batch_insert_image_frame(video.image_frame)
            .await?
            .into_iter()
            .map(|id| id.id_with_table())
            .collect::<Vec<String>>();

        let audio_frame_ids = self
            .batch_insert_audio_frame(video.audio_frame)
            .await?
            .into_iter()
            .map(|id| id.id_with_table())
            .collect::<Vec<String>>();
        debug!("insert audio frame ids: {:?}", audio_frame_ids);

        let image_frame = if image_frame_ids.is_empty() {
            "image_frame: []".to_string()
        } else {
            format!("image_frame: [{}]", image_frame_ids.join(", "))
        };
        debug!("image frame: {:?}", image_frame);

        let audio_frame = if audio_frame_ids.is_empty() {
            "audio_frame: []".to_string()
        } else {
            format!("audio_frame: [{}]", audio_frame_ids.join(", "))
        };
        debug!("audio frame: {:?}", audio_frame);

        let sql = format!(
            "(CREATE ONLY video CONTENT {{ {}, {} }}).id",
            image_frame, audio_frame
        );

        let mut res = self.client.query(&sql).await?;
        match res.take::<Option<Thing>>(0)? {
            Some(id) => {
                let id: ID = id.into();
                self.create_contain_relation(
                    &id.id_with_table(),
                    concat_arrays!(image_frame_ids, audio_frame_ids)
                        .iter()
                        .map(|id| id.as_str())
                        .collect(),
                )
                .await?;
                let payload = self.create_payload(file_identifier.into()).await?;
                self.create_with_relation(&id, &payload).await?;
                Ok(id)
            }
            None => Err(anyhow::anyhow!("Failed to insert video")),
        }
    }

    pub async fn insert_web_page(
        &self,
        web_page: WebPageModel,
        file_identifier: String,
    ) -> anyhow::Result<ID> {
        let page_ids = self
            .batch_insert_page(web_page.page)
            .await?
            .into_iter()
            .map(|id| id.id_with_table())
            .collect::<Vec<String>>();
        if page_ids.is_empty() {
            bail!("Failed to insert web page, page is empty");
        }
        let sql = format!(
            "(CREATE ONLY web CONTENT {{ page: [{}] }}).id",
            page_ids.join(", ")
        );
        let mut res = self.client.query(&sql).await?;
        match res.take::<Option<Thing>>(0)? {
            Some(id) => {
                let id: ID = id.into();
                self.create_contain_relation(
                    &id.id_with_table(),
                    page_ids.iter().map(|id| id.as_str()).collect(),
                )
                .await?;
                let payload_id = self.create_payload(file_identifier.into()).await?;
                self.create_with_relation(&id, &payload_id).await?;
                Ok(id)
            }
            None => Err(anyhow::anyhow!("Failed to insert web page")),
        }
    }

    pub async fn insert_document(
        &self,
        document: DocumentModel,
        file_identifier: String,
    ) -> anyhow::Result<ID> {
        let page_ids = self
            .batch_insert_page(document.page)
            .await?
            .into_iter()
            .map(|id| id.id_with_table())
            .collect::<Vec<String>>();
        if page_ids.is_empty() {
            bail!("Failed to insert document, page is empty");
        }
        let sql = format!(
            "(CREATE ONLY document CONTENT {{ page: [{}] }}).id",
            page_ids.join(", ")
        );
        let mut res = self.client.query(&sql).await?;
        match res.take::<Option<Thing>>(0)? {
            Some(id) => {
                let id: ID = id.into();
                self.create_contain_relation(
                    &id.id_with_table(),
                    page_ids.iter().map(|id| id.as_str()).collect(),
                )
                .await?;
                let payload_id = self.create_payload(file_identifier.into()).await?;
                self.create_with_relation(&id, &payload_id).await?;
                Ok(id)
            }
            None => Err(anyhow::anyhow!("Failed to insert document")),
        }
    }

    /// use for test
    pub async fn upsert(&self, id: &ID, set_clause: &str) -> anyhow::Result<()> {
        let mut resp = self
            .client
            .query(format!("UPSERT {} SET {};", id.id_with_table(), set_clause))
            .await?;
        check_db_error_from_resp!(resp).map_err(|errors_map| {
            error!("upsert errors: {:?}", errors_map);
            anyhow::anyhow!("Failed to upsert, errors: {:?}", errors_map)
        })?;
        Ok(())
    }
}

/// inner functions
impl DB {
    pub(crate) async fn insert_text(&self, text: TextModel) -> anyhow::Result<ID> {
        let mut resp = self
            .client
            .query(
                "
            (CREATE ONLY text CONTENT {
                data: $data,
                vector: $vector,
                en_data: $en_data,
                en_vector: $en_vector
            }).id",
            )
            .bind(text)
            .await?;

        check_db_error_from_resp!(resp).map_err(|errors_map| {
            error!("insert text errors: {:?}", errors_map);
            anyhow::anyhow!("Failed to insert text, errors: {:?}", errors_map)
        })?;

        resp.take::<Option<Thing>>(0)?
            .map(|x| Ok(x.into()))
            .unwrap_or_else(|| Err(anyhow::anyhow!("Failed to insert text")))
    }

    async fn create_payload(&self, payload: PayloadModel) -> anyhow::Result<ID> {
        let mut resp = self
            .client
            .query(
                "
                (CREATE ONLY payload CONTENT {
                    file_identifier: $file_identifier,
                    url: $url
                }).id",
            )
            .bind(payload)
            .await?;

        check_db_error_from_resp!(resp).map_err(|errors_map| {
            error!("create payload errors: {:?}", errors_map);
            anyhow::anyhow!("Failed to create payload, errors: {:?}", errors_map)
        })?;

        match resp.take::<Option<Thing>>(0)? {
            Some(id) => Ok(id.into()),
            None => Err(anyhow::anyhow!("Failed to create payload")),
        }
    }

    async fn insert_audio_frame(&self, frame: AudioFrameModel) -> anyhow::Result<ID> {
        let ids = self
            .batch_insert_text(frame.data)
            .await?
            .into_iter()
            .map(|id| id.id_with_table())
            .collect::<Vec<_>>();
        debug!("insert text ids: {:?}", ids);
        if ids.is_empty() {
            bail!("Failed to insert audio frame");
        }
        let create_audio_frame_sql = format!(
            "(CREATE ONLY audio_frame CONTENT {{ data: [{}], start_timestamp: {}, end_timestamp: {} }}).id",
            ids.join(", "),
            frame.start_timestamp,
            frame.end_timestamp
        );
        let mut res = self.client.query(create_audio_frame_sql).await?;
        match res.take::<Option<Thing>>(0)? {
            Some(id) => {
                let id: ID = id.into();
                self.create_contain_relation(
                    &id.id_with_table(),
                    ids.iter().map(|id| id.as_str()).collect(),
                )
                .await?;
                Ok(id.into())
            }
            None => Err(anyhow::anyhow!("Failed to insert audio frame")),
        }
    }

    async fn insert_image_frame(&self, frame: ImageFrameModel) -> anyhow::Result<ID> {
        let ids = self
            .batch_insert_image(frame.data)
            .await?
            .into_iter()
            .map(|id| id.id_with_table())
            .collect::<Vec<String>>();
        if ids.is_empty() {
            bail!("Failed to insert image frame");
        }
        let create_image_frame_sql = format!(
            "(CREATE ONLY image_frame CONTENT {{ data: [{}], start_timestamp: {}, end_timestamp: {} }}).id",
            ids.join(", "),
            frame.start_timestamp,
            frame.end_timestamp
        );
        let mut res = self.client.query(create_image_frame_sql).await?;
        match res.take::<Option<Thing>>(0)? {
            Some(id) => {
                let id: ID = id.into();
                self.create_contain_relation(
                    &id.id_with_table(),
                    ids.iter().map(|id| id.as_str()).collect(),
                )
                .await?;
                Ok(id.into())
            }
            None => Err(anyhow::anyhow!("Failed to insert image frame")),
        }
    }

    async fn insert_page(&self, data: PageModel) -> anyhow::Result<ID> {
        let text_ids = self
            .batch_insert_text(data.text)
            .await?
            .into_iter()
            .map(|id| id.id_with_table())
            .collect::<Vec<String>>();
        let image_ids = self
            .batch_insert_image(data.image)
            .await?
            .into_iter()
            .map(|id| id.id_with_table())
            .collect::<Vec<String>>();
        let image_frame = if text_ids.is_empty() {
            "text: []".to_string()
        } else {
            format!("text: [{}]", text_ids.join(", "))
        };

        let audio_frame = if image_ids.is_empty() {
            "image: []".to_string()
        } else {
            format!("image: [{}]", image_ids.join(", "))
        };

        let sql = format!(
            "(CREATE ONLY page CONTENT {{ {}, {}, start_index: {}, end_index: {} }}).id",
            image_frame, audio_frame, data.start_index, data.end_index
        );
        let mut res = self.client.query(&sql).await?;
        match res.take::<Option<Thing>>(0)? {
            Some(id) => {
                let id: ID = id.into();
                self.create_contain_relation(
                    &id.id_with_table(),
                    concat_arrays!(text_ids, image_ids)
                        .to_vec()
                        .iter()
                        .map(|id| id.as_str())
                        .collect(),
                )
                .await?;
                Ok(id)
            }
            None => Err(anyhow::anyhow!("Failed to insert page")),
        }
    }

    async fn batch_insert_audio_frame(
        &self,
        frames: Vec<AudioFrameModel>,
    ) -> anyhow::Result<Vec<ID>> {
        let futures = frames
            .into_iter()
            .map(|frame| self.insert_audio_frame(frame))
            .collect::<Vec<_>>();

        collect_async_results!(futures)
    }

    async fn batch_insert_text(&self, texts: Vec<TextModel>) -> anyhow::Result<Vec<ID>> {
        let futures = texts
            .into_iter()
            .map(|text| self.insert_text(text))
            .collect::<Vec<_>>();
        collect_async_results!(futures)
    }

    async fn batch_insert_image(&self, images: Vec<ImageModel>) -> anyhow::Result<Vec<ID>> {
        let futures = images
            .into_iter()
            .map(|image| self.insert_image(image, None))
            .collect::<Vec<_>>();
        collect_async_results!(futures)
    }

    async fn batch_insert_image_frame(
        &self,
        frames: Vec<ImageFrameModel>,
    ) -> anyhow::Result<Vec<ID>> {
        let futures = frames
            .into_iter()
            .map(|frame| self.insert_image_frame(frame))
            .collect::<Vec<_>>();

        collect_async_results!(futures)
    }

    async fn batch_insert_page(&self, pages: Vec<PageModel>) -> anyhow::Result<Vec<ID>> {
        let futures = pages
            .into_iter()
            .map(|page| self.insert_page(page))
            .collect::<Vec<_>>();
        collect_async_results!(futures)
    }
}

/// create relation
impl DB {
    async fn create_with_relation(
        &self,
        relation_in: &ID,
        relation_out: &ID,
    ) -> anyhow::Result<()> {
        let sql = format!(
            "RELATE {} -> with -> {};",
            relation_in.id_with_table(),
            relation_out.id_with_table(),
        );
        self.client.query(&sql).await?;
        Ok(())
    }

    async fn create_contain_relation(
        &self,
        relation_in: &str,
        relation_outs: Vec<&str>,
    ) -> anyhow::Result<()> {
        let sql = format!(
            "RELATE {} -> contains -> [{}];",
            relation_in,
            relation_outs.join(", "),
        );
        self.client.query(&sql).await?;
        Ok(())
    }
}

#[allow(unused_imports, dead_code)]
mod test {
    use crate::check_db_error_from_resp;
    use crate::db::entity::TextEntity;
    use crate::db::model::id::{ID, TB};
    use crate::db::model::{ImageModel, TextModel};
    use crate::db::shared::test::{
        fake_audio_model, fake_document, fake_file_identifier, fake_image_model, fake_page_model,
        fake_video_model, fake_web_page_model, gen_vector, setup,
    };
    use crate::db::DB;
    use itertools::Itertools;
    use std::process::id;
    use test_log::test;

    #[test(tokio::test)]
    async fn test_insert_text() {
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
        let db = setup(None).await;
        let _ = db
            .insert_image(fake_image_model(), Some(fake_file_identifier()))
            .await;
    }

    #[test(tokio::test)]
    async fn test_insert_audio() {
        let db = setup(None).await;
        let id = db
            .insert_audio(fake_audio_model(), fake_file_identifier())
            .await
            .unwrap();
        assert_eq!(id.tb(), &TB::Audio);
    }

    #[test(tokio::test)]
    async fn test_insert_video() {
        let db = setup(None).await;
        let id = db
            .insert_video(fake_video_model(), fake_file_identifier())
            .await
            .unwrap();
        assert_eq!(id.tb(), &TB::Video);
    }

    #[test(tokio::test)]
    async fn test_insert_page() {
        let db = setup(None).await;
        let id = db.insert_page(fake_page_model()).await.unwrap();
        assert_eq!(id.tb(), &TB::Page);
    }

    #[test(tokio::test)]
    async fn test_insert_web_page() {
        let db = setup(None).await;
        let id = db
            .insert_web_page(fake_web_page_model(), fake_file_identifier())
            .await
            .unwrap();
        assert_eq!(id.tb(), &TB::Web);
    }

    #[test(tokio::test)]
    async fn test_insert_document() {
        let db = setup(None).await;
        let id = db
            .insert_document(fake_document(), fake_file_identifier())
            .await
            .unwrap();
        assert_eq!(id.tb(), &TB::Document);
    }

    #[test(tokio::test)]
    async fn test_upsert() {
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
