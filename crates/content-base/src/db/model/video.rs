use crate::db::model::{id::ID, AudioFrameModel, ImageModel};
use educe::Educe;
use serde::Serialize;

#[derive(Serialize, Educe, Clone)]
#[educe(Debug)]
pub struct ImageFrameModel {
    pub id: Option<ID>,

    #[educe(Debug(ignore))]
    pub data: Vec<ImageModel>,
    pub start_timestamp: f32,
    pub end_timestamp: f32,
}

#[derive(Serialize, Educe, Clone)]
#[educe(Debug)]
pub struct VideoModel {
    pub id: Option<ID>,

    #[educe(Debug(ignore))]
    pub image_frame: Vec<ImageFrameModel>,

    #[educe(Debug(ignore))]
    pub audio_frame: Vec<AudioFrameModel>,
}

const IMAGE_FRAME_CREATE_STATEMENT: &'static str = r#"
(CREATE ONLY image_frame CONTENT {{
    data: $images,
    start_timestamp: $start_timestamp,
    end_timestamp: $end_timestamp
}}).id
"#;

impl ImageFrameModel {
    pub async fn create_only<T>(
        client: &surrealdb::Surreal<T>,
        image_frame_model: &Self,
    ) -> anyhow::Result<surrealdb::sql::Thing>
    where
        T: surrealdb::Connection,
    {
        let image_records = ImageModel::create_batch(client, &image_frame_model.data).await?;
        if image_records.is_empty() {
            anyhow::bail!("Failed to insert frame images, images is empty");
        }
        let mut resp = client
            .query(IMAGE_FRAME_CREATE_STATEMENT)
            .bind(("images", image_records.clone()))
            .bind(image_frame_model.clone())
            .await?;
        if let Err(errors_map) = crate::check_db_error_from_resp!(resp) {
            anyhow::bail!("Failed to insert image frame, errors: {:?}", errors_map);
        };
        let Some(image_frame_record) = resp.take::<Option<surrealdb::sql::Thing>>(0)? else {
            anyhow::bail!("Failed to insert image frame, no id returned");
        };
        client
            .query("RELATE $relation_in -> contains -> $relation_outs;")
            .bind(("relation_in", image_frame_record.clone()))
            .bind(("relation_outs", image_records.clone()))
            .await?;
        Ok(image_frame_record)
    }

    pub async fn create_batch<T>(
        client: &surrealdb::Surreal<T>,
        image_frame_models: &Vec<Self>,
    ) -> anyhow::Result<Vec<surrealdb::sql::Thing>>
    where
        T: surrealdb::Connection,
    {
        let futures = image_frame_models
            .into_iter()
            .map(|image_frame_model| Self::create_only(client, image_frame_model))
            .collect::<Vec<_>>();
        let results = crate::collect_async_results!(futures);
        results
    }

    pub fn table() -> &'static str {
        "image_frame"
    }
}

const VIDEO_CREATE_STATEMENT: &'static str = r#"
(CREATE ONLY video CONTENT {{
    image_frame: $image_frames,
    audio_frame: $audio_frames
}}).id
"#;

impl VideoModel {
    pub async fn create_only<T>(
        client: &surrealdb::Surreal<T>,
        video_model: &Self,
    ) -> anyhow::Result<surrealdb::sql::Thing>
    where
        T: surrealdb::Connection,
    {
        let image_frame_records =
            ImageFrameModel::create_batch(client, &video_model.image_frame).await?;
        let audio_frame_records =
            AudioFrameModel::create_batch(client, &video_model.audio_frame).await?;
        let mut resp = client
            .query(VIDEO_CREATE_STATEMENT)
            .bind(("image_frames", image_frame_records.clone()))
            .bind(("audio_frames", audio_frame_records.clone()))
            .await?;
        if let Err(errors_map) = crate::check_db_error_from_resp!(resp) {
            anyhow::bail!("Failed to insert video, errors: {:?}", errors_map);
        };
        let Some(video_record) = resp.take::<Option<surrealdb::sql::Thing>>(0)? else {
            anyhow::bail!("Failed to insert video, no id returned");
        };
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

    pub fn table() -> &'static str {
        "video"
    }
}
