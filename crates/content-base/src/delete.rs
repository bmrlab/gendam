use crate::ContentBase;
use qdrant_client::qdrant::{Condition, DeletePointsBuilder, Filter};

pub struct DeletePayload {
    file_identifier: String,
}

impl DeletePayload {
    pub fn new(file_identifier: &str) -> Self {
        Self {
            file_identifier: file_identifier.to_string(),
        }
    }
}

impl ContentBase {
    pub async fn delete(&self, payload: DeletePayload) -> anyhow::Result<()> {
        // delete in database
        match self.qdrant.list_collections().await {
            std::result::Result::Ok(collections) => {
                for collection in collections.collections.iter() {
                    self.qdrant
                        .delete_points(
                            DeletePointsBuilder::new(&collection.name)
                                .points(Filter::must([Condition::matches(
                                    "file_identifier",
                                    payload.file_identifier.to_string(),
                                )]))
                                .wait(true),
                        )
                        .await?;
                }
            }
            _ => {
                tracing::warn!("failed to list collections");
            }
        }

        // delete in file system
        self.ctx()
            .delete_artifacts(&payload.file_identifier)
            .await?;

        Ok(())
    }
}
