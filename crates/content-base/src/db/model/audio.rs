use super::{id::ID, text::TextModel, ModelCreate, ModelDelete};
use async_trait::async_trait;
use educe::Educe;
use serde::Serialize;

#[derive(Serialize, Educe, Clone)]
#[educe(Debug)]
pub struct AudioFrameModel {
    pub id: Option<ID>,
    pub start_timestamp: i64,
    pub end_timestamp: i64,
}

const AUDIO_FRAME_CREATE_STATEMENT: &'static str = r#"
(CREATE ONLY audio_frame CONTENT {{
    start_timestamp: $start_timestamp,
    end_timestamp: $end_timestamp
}}).id
"#;

#[async_trait]
impl<T> ModelCreate<T, (Self, Vec<TextModel>)> for AudioFrameModel
where
    T: surrealdb::Connection,
{
    async fn create_only(
        client: &surrealdb::Surreal<T>,
        (audio_frame, transcript_texts): &(Self, Vec<TextModel>),
    ) -> anyhow::Result<surrealdb::sql::Thing> {
        let text_records = TextModel::create_batch(client, transcript_texts).await?;
        if text_records.is_empty() {
            anyhow::bail!("Failed to insert frame texts, texts is empty");
        }
        let mut resp = client
            .query(AUDIO_FRAME_CREATE_STATEMENT)
            .bind(audio_frame.clone())
            .await?;
        if let Err(errors_map) = crate::check_db_error_from_resp!(resp) {
            anyhow::bail!("Failed to insert audio frame, errors: {:?}", errors_map);
        };
        let Some(audio_frame_record) = resp.take::<Option<surrealdb::sql::Thing>>(0)? else {
            anyhow::bail!("Failed to insert audio frame, no id returned");
        };
        tracing::debug!(id=%audio_frame_record, "Audio frame created in surrealdb");
        client
            .query("RELATE $relation_in -> contains -> $relation_outs;")
            .bind(("relation_in", audio_frame_record.clone()))
            .bind(("relation_outs", text_records.clone()))
            .await?;
        Ok(audio_frame_record)
    }
}

impl AudioFrameModel {
    pub fn table() -> &'static str {
        "audio_frame"
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct AudioModel {
    pub id: Option<ID>,
}

const AUDIO_CREATE_STATEMENT: &'static str = r#"
(CREATE ONLY audio CONTENT {{}}).id
"#;

#[async_trait]
impl<T> ModelCreate<T, (Self, Vec<(AudioFrameModel, Vec<TextModel>)>)> for AudioModel
where
    T: surrealdb::Connection,
{
    async fn create_only(
        client: &surrealdb::Surreal<T>,
        (_audio_model, audio_frames): &(Self, Vec<(AudioFrameModel, Vec<TextModel>)>),
    ) -> anyhow::Result<surrealdb::sql::Thing> {
        let audio_frame_records = AudioFrameModel::create_batch(client, audio_frames).await?;
        if audio_frame_records.is_empty() {
            anyhow::bail!("Failed to insert audio frames, frames is empty");
        }
        let mut resp = client.query(AUDIO_CREATE_STATEMENT).await?;
        if let Err(errors_map) = crate::check_db_error_from_resp!(resp) {
            anyhow::bail!("Failed to insert audio, errors: {:?}", errors_map);
        };
        let Some(audio_record) = resp.take::<Option<surrealdb::sql::Thing>>(0)? else {
            anyhow::bail!("Failed to insert audio, no id returned");
        };
        tracing::debug!(id=%audio_record, "Audio created in surrealdb");
        client
            .query("RELATE $relation_in -> contains -> $relation_outs;")
            .bind(("relation_in", audio_record.clone()))
            .bind(("relation_outs", audio_frame_records.clone()))
            .await?;
        Ok(audio_record)
    }
}

const AUDIO_DELETE_STATEMENT: &'static str = r#"
LET $v = (
    SELECT
        ->contains->audio_frame AS audio_frames,
        ->contains->audio_frame->contains->text AS texts,
        ->with->payload AS payload,
        id
    FROM ONLY $record
);
let $ids = array::flatten([$v.texts, $v.audio_frames, $v.payload, $v.id]);
DELETE $ids;
"#;

#[async_trait]
impl<T> ModelDelete<T> for AudioModel
where
    T: surrealdb::Connection,
{
    async fn delete_cascade(
        client: &surrealdb::Surreal<T>,
        record: &surrealdb::sql::Thing,
    ) -> anyhow::Result<()> {
        let mut resp = client
            .query(AUDIO_DELETE_STATEMENT)
            .bind(("record", record.clone()))
            .await?;
        if let Err(errors_map) = crate::check_db_error_from_resp!(resp) {
            anyhow::bail!("Failed to delete audio, errors: {:?}", errors_map);
        };
        Ok(())
    }
}

impl AudioModel {
    pub fn table() -> &'static str {
        "audio"
    }
}
