use super::ContentBase;
use crate::{
    metadata::ContentMetadata, record::{TaskRecord, TaskRunRecord}, ContentTask, ContentTaskType
};
use std::path::Path;
use storage::Storage;

impl ContentBase {
    fn task_record_path(&self, file_identifier: &str) -> impl AsRef<Path> {
        self.artifacts_dir(file_identifier).join("artifacts.json")
    }

    pub async fn save_task_record(
        &self,
        file_identifier: &str,
        task_record: &TaskRecord,
    ) -> anyhow::Result<()> {
        self.write(
            self.task_record_path(file_identifier)
                .as_ref()
                .to_path_buf(),
            serde_json::to_string(&task_record)?.into(),
        )
        .await?;
        Ok(())
    }

    pub async fn task_record(&self, file_identifier: &str) -> TaskRecord {
        match self.read_to_string(
            self.task_record_path(file_identifier)
                .as_ref()
                .to_path_buf(),
        ) {
            Ok(record) => match serde_json::from_str::<TaskRecord>(&record) {
                Ok(record) => record,
                Err(_) => TaskRecord::new(file_identifier, None),
            },
            _ => TaskRecord::new(file_identifier, None),
        }
    }

    pub async fn set_metadata(
        &self,
        file_identifier: &str,
        metadata: &ContentMetadata,
    ) -> anyhow::Result<()> {
        let record = self.task_record(file_identifier).await;
        let record = record.with_metadata(metadata);
        self.save_task_record(file_identifier, &record).await?;

        Ok(())
    }

    pub async fn create_task(
        &self,
        file_identifier: &str,
        task_type: &ContentTaskType,
    ) -> anyhow::Result<TaskRunRecord> {
        let record = self.task_record(file_identifier).await;
        let mut tasks = record.tasks().clone();

        if !tasks.contains_key(task_type) {
            tasks.insert(task_type.clone(), vec![]);
        }

        let mut task_run_record = TaskRunRecord::new(task_type);

        // FIXME Add task dependencies according to map
        // task_run_record.with_deps()
        let task_parameters = task_type.task_parameters(self);
        let task_output = task_type.task_output(&task_run_record).await?;
        task_run_record.with_output(&task_output);
        task_run_record.with_parameters(&task_parameters);

        let runs = tasks.get_mut(task_type).expect("data has been inserted");
        runs.push(task_run_record.clone());

        let record = record.with_tasks(&tasks);
        self.save_task_record(file_identifier, &record).await?;

        Ok(task_run_record)
    }

    pub async fn set_task_run_record(
        &self,
        file_identifier: &str,
        task_type: &ContentTaskType,
        task_run_record: &TaskRunRecord,
    ) -> anyhow::Result<()> {
        let record = self.task_record(file_identifier).await;
        let mut tasks = record.tasks().clone();
        let runs = tasks
            .get_mut(task_type)
            .ok_or(anyhow::anyhow!("task record not found"))?;
        let index = runs
            .iter()
            .position(|task| task.id() == task_run_record.id())
            .ok_or(anyhow::anyhow!("task run record not found"))?;
        runs[index] = task_run_record.clone();
        let record = record.with_tasks(&tasks);
        self.save_task_record(file_identifier, &record).await?;

        Ok(())
    }

    /// Get the lated task run record of given task type.
    pub async fn task_run_record(
        &self,
        file_identifier: &str,
        task_type: &ContentTaskType,
    ) -> anyhow::Result<TaskRunRecord> {
        let record = self.task_record(file_identifier).await;
        record
            .tasks()
            .get(task_type)
            .and_then(|v| v.last())
            .cloned()
            .ok_or(anyhow::anyhow!("task run record not found"))
    }
}
