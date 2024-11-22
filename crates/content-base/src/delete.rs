use crate::ContentBase;
use async_recursion::async_recursion;
use content_base_context::ContentBaseCtx;
use content_base_task::{ContentTask, ContentTaskType, FileInfo, TaskRecord};
use std::path::PathBuf;
use tracing::info;

pub struct DeletePayload {
    file_identifier: String,
    keep_search_indexes: bool,
    keep_completed_tasks: bool,
}

impl DeletePayload {
    pub fn new(file_identifier: &str) -> Self {
        Self {
            file_identifier: file_identifier.to_string(),
            keep_search_indexes: false,
            keep_completed_tasks: false,
        }
    }

    pub fn keep_search_indexes(mut self, keep_search_indexes: bool) -> Self {
        self.keep_search_indexes = keep_search_indexes;
        self
    }

    pub fn keep_completed_tasks(mut self, keep_completed_tasks: bool) -> Self {
        self.keep_completed_tasks = keep_completed_tasks;
        self
    }
}

impl ContentBase {
    pub async fn delete(&self, payload: DeletePayload) -> anyhow::Result<()> {
        self.delete_search_indexes(&payload).await?;
        self.delete_artifacts(&payload).await?;
        Ok(())
    }

    /// 删除 surrealdb 中的索引
    async fn delete_search_indexes(&self, payload: &DeletePayload) -> anyhow::Result<()> {
        if payload.keep_search_indexes {
            tracing::debug!("keep_search_indexes is true, skip deleting search indexes");
            return Ok(());
        }
        self.surrealdb_client
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
    /// 这里只删除 ../core.rs 中 tasks 方法里列出的任务相关的所有数据也
    ///   - 就是当前处理逻辑有关的任务
    ///   - 对应任务类型的不同执行参数的所有执行记录都会被删除
    /// 其他的任务（如果有，比如旧版的任务），需要其他地方另外删除，这里不处理
    async fn delete_artifacts(&self, payload: &DeletePayload) -> anyhow::Result<()> {
        let task_record = TaskRecord::from_content_base(&payload.file_identifier, &self.ctx).await;

        let file_info = FileInfo {
            file_identifier: payload.file_identifier.clone(),
            file_full_path_on_disk: PathBuf::new(), // this filed is not used in delete
        };

        let tasks = Self::get_content_processing_tasks(task_record.metadata());
        for (task, _) in tasks {
            delete_task(&file_info, &task, &self.ctx, payload.keep_completed_tasks).await;
        }

        Ok(())
    }
}

#[async_recursion]
async fn delete_task(
    file_info: &FileInfo,
    task_type: &ContentTaskType,
    ctx: &ContentBaseCtx,
    keep_completed_tasks: bool,
) -> bool {
    let task_type: ContentTaskType = task_type.into(); // TODO: 这个 into 感觉没必要啊 ?
    let task_record = TaskRecord::from_content_base(&file_info.file_identifier, &ctx).await;
    let is_completed = task_record
        .target_run(ctx, &task_type)
        .map_or(false, |r| r.is_completed());

    let mut any_dep_is_deleted = false;
    for dep_task_type in task_type.task_dependencies().iter() {
        let is_deleted = delete_task(file_info, dep_task_type, ctx, keep_completed_tasks).await;
        any_dep_is_deleted = any_dep_is_deleted || is_deleted;
    }

    // TODO: 这段代码用来强制删除一些任务的结果，主要用于测试期间，后面要支持任务的 version 在参数里。
    // 记得要放在这个位置，不能放在上面的循环之前，依赖的任务依然保持原有逻辑，不受影响，除非在下面专门指定。
    // let keep_completed_tasks = match task_type {
    //     ContentTaskType::Video(
    //         VideoTaskType::TransChunkSum(_) | VideoTaskType::TransChunkSumEmbed(_),
    //     ) => false,
    //     _ => keep_completed_tasks,
    // };

    if keep_completed_tasks && is_completed && !any_dep_is_deleted {
        // 如果任务已经完成，且没有依赖任务被删除，则不删除
        tracing::info!(
            "Task {} of {} is completed and will not be deleted",
            task_type,
            file_info.file_identifier
        );
        return false;
    }

    if let Err(e) = task_type.delete_artifacts(&file_info, ctx).await {
        tracing::warn!("failed to delete artifacts: {}", e);
    }

    true
}
