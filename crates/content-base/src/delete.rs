use crate::ContentBase;
use async_recursion::async_recursion;
use content_base_context::ContentBaseCtx;
use content_base_task::{ContentTask, ContentTaskType, FileInfo, TaskRecord};
use qdrant_client::qdrant::{Condition, DeletePointsBuilder, Filter};
use std::path::PathBuf;

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

        let task_record = TaskRecord::from_content_base(&payload.file_identifier, &self.ctx).await;

        let file_info = FileInfo {
            file_identifier: payload.file_identifier.clone(),
            file_path: PathBuf::new(), // this filed is not used in delete
        };

        let tasks = Self::tasks(task_record.metadata());
        for (task, _) in tasks {
            delete_task(&file_info, &task, self.ctx()).await;
        }

        Ok(())
    }
}

#[async_recursion]
async fn delete_task(
    file_info: &FileInfo,
    task_type: &ContentTaskType,
    ctx: &ContentBaseCtx,
) {
    let task_type: ContentTaskType = task_type.into();
    let deps = task_type.task_dependencies();

    for dep in deps {
        delete_task(file_info, &dep, ctx).await;
    }

    if let Err(e) = task_type.delete_artifacts(&file_info, ctx).await {
        tracing::warn!("failed to delete artifacts: {}", e);
    }
}
