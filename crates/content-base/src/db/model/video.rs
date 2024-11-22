use super::{
    audio::AudioFrameModel, id::ID, image::ImageModel, text::TextModel, ModelCreate, ModelDelete,
};
use async_trait::async_trait;
use educe::Educe;
use serde::Serialize;

#[derive(Serialize, Educe, Clone)]
#[educe(Debug)]
pub struct ImageFrameModel {
    pub id: Option<ID>,
    // #[educe(Debug(ignore))]
    // pub data: Vec<ImageModel>,
    pub start_timestamp: i64,
    pub end_timestamp: i64,
}

const IMAGE_FRAME_CREATE_STATEMENT: &'static str = r#"
(CREATE ONLY image_frame CONTENT {{
    -- data: $images,
    start_timestamp: $start_timestamp,
    end_timestamp: $end_timestamp
}}).id
"#;

#[async_trait]
impl<T> ModelCreate<T, (Self, Vec<ImageModel>)> for ImageFrameModel
where
    T: surrealdb::Connection,
{
    async fn create_only(
        client: &surrealdb::Surreal<T>,
        (image_frame, frame_images): &(Self, Vec<ImageModel>),
    ) -> anyhow::Result<surrealdb::sql::Thing> {
        let image_records = ImageModel::create_batch(client, frame_images).await?;
        if image_records.is_empty() {
            anyhow::bail!("Failed to insert frame images, images is empty");
        }
        let mut resp = client
            .query(IMAGE_FRAME_CREATE_STATEMENT)
            // .bind(("images", image_records.clone()))
            .bind(image_frame.clone())
            .await?;
        if let Err(errors_map) = crate::check_db_error_from_resp!(resp) {
            anyhow::bail!("Failed to insert image frame, errors: {:?}", errors_map);
        };
        let Some(image_frame_record) = resp.take::<Option<surrealdb::sql::Thing>>(0)? else {
            anyhow::bail!("Failed to insert image frame, no id returned");
        };
        tracing::debug!(id=%image_frame_record, "Image frame created in surrealdb");
        client
            .query("RELATE $relation_in -> contains -> $relation_outs;")
            .bind(("relation_in", image_frame_record.clone()))
            .bind(("relation_outs", image_records.clone()))
            .await?;
        Ok(image_frame_record)
    }
}

impl ImageFrameModel {
    pub fn table() -> &'static str {
        "image_frame"
    }
}

#[derive(Serialize, Educe, Clone)]
#[educe(Debug)]
pub struct VideoModel {
    pub id: Option<ID>,
}

const VIDEO_CREATE_STATEMENT: &'static str = r#"
(CREATE ONLY video CONTENT {{}}).id
"#;

#[async_trait]
impl<T>
    ModelCreate<
        T,
        (
            Self,
            Vec<(ImageFrameModel, Vec<ImageModel>)>,
            Vec<(AudioFrameModel, Vec<TextModel>)>,
        ),
    > for VideoModel
where
    T: surrealdb::Connection,
{
    async fn create_only(
        client: &surrealdb::Surreal<T>,
        (_video, image_frames, audio_frames): &(
            Self,
            Vec<(ImageFrameModel, Vec<ImageModel>)>,
            Vec<(AudioFrameModel, Vec<TextModel>)>,
        ),
    ) -> anyhow::Result<surrealdb::sql::Thing> {
        let image_frame_records = ImageFrameModel::create_batch(client, image_frames).await?;
        let audio_frame_records = AudioFrameModel::create_batch(client, audio_frames).await?;
        let mut resp = client.query(VIDEO_CREATE_STATEMENT).await?;
        if let Err(errors_map) = crate::check_db_error_from_resp!(resp) {
            anyhow::bail!("Failed to insert video, errors: {:?}", errors_map);
        };
        let Some(video_record) = resp.take::<Option<surrealdb::sql::Thing>>(0)? else {
            anyhow::bail!("Failed to insert video, no id returned");
        };
        tracing::debug!(id=%video_record, "Video created in surrealdb");
        let all_frames = Vec::new()
            .into_iter()
            .chain(image_frame_records.into_iter())
            .chain(audio_frame_records.into_iter())
            .collect::<Vec<_>>();
        client
            .query("RELATE $relation_in -> contains -> $relation_outs;")
            .bind(("relation_in", video_record.clone()))
            .bind(("relation_outs", all_frames))
            .await?;
        Ok(video_record)
    }
}

const VIDEO_DELETE_STATEMENT: &'static str = r#"
LET $v = (
    SELECT
        ->contains->image_frame AS image_frames,
        ->contains->image_frame->contains->image AS images,
        ->contains->audio_frame AS audio_frames,
        ->contains->audio_frame->contains->text AS texts,
        ->with->payload AS payload,
        id
    FROM ONLY $record
);
let $ids = array::flatten([$v.images, $v.texts, $v.image_frames, $v.audio_frames, $v.payload, $v.id]);
DELETE $ids;
"#;

#[async_trait]
impl<T> ModelDelete<T> for VideoModel
where
    T: surrealdb::Connection,
{
    async fn delete_cascade(
        client: &surrealdb::Surreal<T>,
        record: &surrealdb::sql::Thing,
    ) -> anyhow::Result<()> {
        let mut resp = client
            .query(VIDEO_DELETE_STATEMENT)
            .bind(("record", record.clone()))
            .await?;
        if let Err(errors_map) = crate::check_db_error_from_resp!(resp) {
            anyhow::bail!("Failed to delete video, errors: {:?}", errors_map);
        };
        Ok(())
    }
}

impl VideoModel {
    pub fn table() -> &'static str {
        "video"
    }
}
