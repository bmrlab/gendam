use crate::ContentBase;
use content_base_pool::{TaskNotification, TaskPool, TaskPriority};
use content_base_task::{
    video::{frame::VideoFrameTask, trans_chunk_sum_embed::VideoTransChunkSumEmbedTask},
    ContentTaskType, FileInfo, TaskRecord,
};
use content_handler::file_metadata;
use content_metadata::ContentMetadata;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::sync::mpsc::{self, Receiver};
use tracing::warn;

#[derive(Serialize, Deserialize)]
pub struct UpsertPayload {
    file_identifier: String,
    file_path: PathBuf,
    metadata: Option<ContentMetadata>,
}

impl UpsertPayload {
    pub fn new(file_identifier: &str, file_path: impl AsRef<Path>) -> Self {
        Self {
            file_identifier: file_identifier.to_string(),
            file_path: file_path.as_ref().to_path_buf(),
            metadata: None,
        }
    }

    pub fn with_metadata(mut self, metadata: ContentMetadata) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

impl ContentBase {
    pub async fn upsert(
        &self,
        payload: UpsertPayload,
    ) -> anyhow::Result<Receiver<TaskNotification>> {
        let task_pool = self.task_pool.clone();
        let file_identifier = &payload.file_identifier.clone();

        let mut task_record = TaskRecord::from_content_base(file_identifier, &self.ctx).await;
        let metadata = match payload.metadata.clone() {
            Some(metadata) => metadata,
            _ => match task_record.metadata() {
                ContentMetadata::Unknown => {
                    file_metadata(&payload.file_path).expect("got file metadata")
                }
                _ => task_record.metadata().clone(),
            },
        };

        let (notification_tx, notification_rx) = mpsc::channel(512);

        if let Err(e) = task_record.set_metadata(&self.ctx, &metadata).await {
            warn!("failed to set metadata: {e:?}");
        }

        tokio::spawn(async move {
            match metadata {
                ContentMetadata::Video(metadata) => {
                    let file_info = FileInfo {
                        file_identifier: payload.file_identifier,
                        file_path: payload.file_path,
                    };

                    if let Err(e) = run_task(
                        &task_pool,
                        &file_info,
                        VideoFrameTask,
                        Some(TaskPriority::Low),
                        Some(notification_tx.clone()),
                    )
                    .await
                    {
                        warn!("failed to add task: {e:?}");
                    }

                    if metadata.audio.is_some() {
                        if let Err(e) = run_task(
                            &task_pool,
                            &file_info,
                            VideoTransChunkSumEmbedTask,
                            Some(TaskPriority::Low),
                            Some(notification_tx.clone()),
                        )
                        .await
                        {
                            warn!("failed to add task: {e:?}");
                        }
                    }
                }
                ContentMetadata::Audio(_metadata) => {
                    todo!()
                }
                ContentMetadata::Unknown => {
                    warn!(
                        "unknown metadata for {}, do not trigger any tasks",
                        payload.file_identifier
                    );
                }
            }
        });

        Ok(notification_rx)
    }
}

async fn run_task(
    task_pool: &TaskPool,
    file_info: &FileInfo,
    task_type: impl Into<ContentTaskType>,
    priority: Option<TaskPriority>,
    notification_tx: Option<mpsc::Sender<TaskNotification>>,
) -> anyhow::Result<()> {
    task_pool
        .add_task(
            &file_info.file_identifier,
            &file_info.file_path,
            task_type.into(),
            priority,
            notification_tx,
        )
        .await?;

    Ok(())
}
