use crate::db::model::{id::ID, TextModel};
use educe::Educe;
use serde::Serialize;

#[derive(Serialize, Educe, Clone)]
#[educe(Debug)]
pub struct AudioFrameModel {
    pub id: Option<ID>,
    pub start_timestamp: f32,
    pub end_timestamp: f32,
}

const AUDIO_FRAME_CREATE_STATEMENT: &'static str = r#"
(CREATE ONLY audio_frame CONTENT {{
    start_timestamp: $start_timestamp,
    end_timestamp: $end_timestamp
}}).id
"#;

impl AudioFrameModel {
    pub async fn create_only<T>(
        client: &surrealdb::Surreal<T>,
        audio_frame_model: &Self,
    ) -> anyhow::Result<surrealdb::sql::Thing>
    where
        T: surrealdb::Connection,
    {
        let text_records = TextModel::create_batch(client, &audio_frame_model.data).await?;
        if text_records.is_empty() {
            anyhow::bail!("Failed to insert frame texts, texts is empty");
        }
        let mut resp = client
            .query(AUDIO_FRAME_CREATE_STATEMENT)
            .bind(audio_frame_model.clone())
            .await?;
        if let Err(errors_map) = crate::check_db_error_from_resp!(resp) {
            anyhow::bail!("Failed to insert audio frame, errors: {:?}", errors_map);
        };
        let Some(audio_frame_record) = resp.take::<Option<surrealdb::sql::Thing>>(0)? else {
            anyhow::bail!("Failed to insert audio frame, no id returned");
        };
        client
            .query("RELATE $relation_in -> contains -> $relation_outs;")
            .bind(("relation_in", audio_frame_record.clone()))
            .bind(("relation_outs", text_records.clone()))
            .await?;
        Ok(audio_frame_record)
    }

    pub async fn create_batch<T>(
        client: &surrealdb::Surreal<T>,
        audio_frame_models: &Vec<Self>,
    ) -> anyhow::Result<Vec<surrealdb::sql::Thing>>
    where
        T: surrealdb::Connection,
    {
        let futures = audio_frame_models
            .into_iter()
            .map(|audio_frame_model| Self::create_only(client, audio_frame_model))
            .collect::<Vec<_>>();
        let results = crate::collect_async_results!(futures);
        results
    }

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

impl AudioModel {
    pub async fn create_only<T>(
        client: &surrealdb::Surreal<T>,
        audio_model: &Self,
    ) -> anyhow::Result<surrealdb::sql::Thing>
    where
        T: surrealdb::Connection,
    {
        let audio_frame_records =
            AudioFrameModel::create_batch(client, &audio_model.audio_frame).await?;
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
        client
            .query("RELATE $relation_in -> contains -> $relation_outs;")
            .bind(("relation_in", audio_record.clone()))
            .bind(("relation_outs", audio_frame_records.clone()))
            .await?;
        Ok(audio_record)
    }

    pub fn table() -> &'static str {
        "audio"
    }
}
