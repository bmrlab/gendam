use crate::ContentBase;
use async_recursion::async_recursion;
use content_base_context::ContentBaseCtx;
use content_base_task::{ContentTask, ContentTaskType, FileInfo, TaskRecord};
use std::path::PathBuf;
use tracing::info;

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
    /// 删除 surrealdb 中的索引
    pub async fn delete_search_indexes(&self, payload: DeletePayload) -> anyhow::Result<()> {
        self.db
            .try_write()?
            .delete_by_file_identifier(&payload.file_identifier)
            .await?;
        info!(
            "Deleted file_identifier: {} in surrealdb",
            payload.file_identifier
        );

        Ok(())
    }

    /// 删除 artifacts 目录中的任务记录
    /// 这里只删除 ../core.rs 中 tasks 方法里列出的任务相关的所有数据，其他的任务（如果有），需要其他地方另外删除，这里不处理
    pub async fn delete_artifacts(&self, payload: DeletePayload) -> anyhow::Result<()> {
        let task_record = TaskRecord::from_content_base(&payload.file_identifier, &self.ctx).await;

        let file_info = FileInfo {
            file_identifier: payload.file_identifier.clone(),
            file_full_path_on_disk: PathBuf::new(), // this filed is not used in delete
        };

        let tasks = Self::tasks(task_record.metadata());
        for (task, _) in tasks {
            delete_task(&file_info, &task, self.ctx()).await;
        }

        Ok(())
    }
}

#[async_recursion]
async fn delete_task(file_info: &FileInfo, task_type: &ContentTaskType, ctx: &ContentBaseCtx) {
    let task_type: ContentTaskType = task_type.into();
    let deps = task_type.task_dependencies();

    for dep in deps {
        delete_task(file_info, &dep, ctx).await;
    }

    if let Err(e) = task_type.delete_artifacts(&file_info, ctx).await {
        tracing::warn!("failed to delete artifacts: {}", e);
    }
}
