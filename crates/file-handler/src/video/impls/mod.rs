pub mod audio;
pub mod caption;
pub mod frame;

use super::VideoHandler;
use crate::{search::payload::SearchPayload, SearchRecordType};
use qdrant_client::qdrant::{
    point_id::PointIdOptions, points_selector::PointsSelectorOneOf, Condition, Filter, PointId,
    PointStruct, PointsSelector,
};
use serde_json::json;

pub(self) fn get_frame_timestamp_from_path(
    path: impl AsRef<std::path::Path>,
) -> anyhow::Result<i64> {
    let file_name = path
        .as_ref()
        .file_name()
        .ok_or(anyhow::anyhow!("invalid path"))?
        .to_string_lossy()
        .to_string();

    let frame_timestamp: i64 = file_name
        .split(".")
        .next()
        .unwrap_or("0")
        .parse()
        .unwrap_or(0);

    Ok(frame_timestamp)
}

impl VideoHandler {
    pub(self) async fn save_text_embedding(
        &self,
        text: &str,
        payload: SearchPayload,
    ) -> anyhow::Result<()> {
        let qdrant = self.library.qdrant_client();

        // if point exists, skip
        match qdrant
            .get_points(
                self.language_collection_name()?,
                None,
                &[PointId {
                    point_id_options: Some(PointIdOptions::Uuid(payload.get_uuid().to_string())),
                }],
                Some(false),
                Some(false),
                None,
            )
            .await
        {
            std::result::Result::Ok(res) if res.result.len() > 0 => {
                return Ok(());
            }
            _ => {}
        }

        let embedding = self
            .text_embedding()?
            .get_texts_embedding_tx()
            .process_single(text.to_string())
            .await?;

        let point = PointStruct::new(
            payload.get_uuid().to_string(),
            embedding,
            json!(payload)
                .try_into()
                .map_err(|_| anyhow::anyhow!("invalid payload"))?,
        );
        qdrant
            .upsert_points(self.language_collection_name()?, None, vec![point], None)
            .await?;

        Ok(())
    }

    pub(self) async fn delete_embedding(
        &self,
        record_type: SearchRecordType,
    ) -> anyhow::Result<()> {
        let file_identifier = self.file_identifier();
        let qdrant = self.library.qdrant_client();

        let points_selector = PointsSelector {
            points_selector_one_of: Some(PointsSelectorOneOf::Filter(Filter::all(vec![
                Condition::matches("file_identifier", file_identifier.to_string()),
                Condition::matches("record_type", record_type.to_string()),
            ]))),
        };

        let collection_name = match record_type {
            SearchRecordType::FrameCaption => self.language_collection_name()?,
            SearchRecordType::Transcript => self.language_collection_name()?,
            SearchRecordType::Frame => self.vision_collection_name()?,
        };

        qdrant
            .delete_points(collection_name.to_string(), None, &points_selector, None)
            .await?;

        Ok(())
    }
}
