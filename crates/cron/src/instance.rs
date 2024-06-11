use std::fmt::{self, Debug};

use async_cron_scheduler::Scheduler;
use chrono::Local;
use uuid::Uuid;

use crate::{error::CronError, task::Task, utils::parse_cron};

pub struct Instance {
    pub tasks: Vec<Task>,
    pub scheduler: Scheduler<Local>,
}

impl Debug for Instance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Instance")
            .field("file", &self.tasks)
            .field("scheduler", &"Scheduler<Local>")
            .finish()
    }
}

impl Instance {
    pub fn init() -> Self {
        let (scheduler, sched_service) = Scheduler::<Local>::launch(tokio::time::sleep);
        tokio::spawn(sched_service);
        Self {
            tasks: Vec::new(),
            scheduler,
        }
    }

    pub fn generate_id(&self) -> Uuid {
        Uuid::new_v4()
    }

    pub async fn create_job(&mut self, new_task: Task) -> Result<(), CronError> {
        let mut task = new_task.clone();
        let schedule = parse_cron(&task.cron)?;
        task.create_job(schedule, &mut self.scheduler).await;
        self.tasks.push(task);
        Ok(())
    }

    pub fn get_all_job_id(&self) -> Vec<Uuid>{
        let mut task_ids = Vec::new();

        for task in &self.tasks {
            task_ids.push(task.id)
        };

        task_ids
    }
    pub async fn delete_job(&mut self, id: Uuid) -> Result<(), CronError> {
        if let Some(index) = self.tasks.iter().position(|task| task.id == id) {
            let task = self.tasks.remove(index);
            if let Some(job_id) = task.job_id {
                self.scheduler.remove(job_id).await;
            }
        };
        Ok(())
    }
}
