use core::fmt;
use futures::future::BoxFuture;
use std::sync::Arc;

use async_cron_scheduler::cron;
use async_cron_scheduler::{Job, JobId, Scheduler};
use chrono::Local;
use uuid::Uuid;

#[derive(Clone)]
pub struct Task {
    pub title: Option<String>,
    pub description: Option<String>,
    pub enabled: bool,
    pub id: Uuid,
    pub job_id: Option<JobId>,
    pub cron: String, // cron 表达式
    pub job_fn: Arc<dyn Fn() -> BoxFuture<'static, ()> + Send + Sync>,
}

impl fmt::Debug for Task {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Task")
            .field("title", &self.title)
            .field("description", &self.description)
            .field("enabled", &self.enabled)
            .field("id", &self.id)
            .field("job_id", &self.job_id)
            .field("cron", &self.cron)
            .finish()
    }
}

impl Task {
    pub async fn create_job(&mut self, cron: cron::Schedule, scheduler: &mut Scheduler<Local>) {
        if self.enabled {
            let job_fn = Arc::clone(&self.job_fn);
            let job = Job::cron_schedule(cron);
            let job_id = scheduler
                .insert(job, move |_id| {
                    // 触发 job_fn
                    let job_future = (job_fn)();
                    tokio::spawn(async move {
                        job_future.await;
                    });
                })
                .await;
            self.job_id = Some(job_id);
            tracing::info!("Create job \"{:?}\" at {:?}", self.title, self.cron);
        }
    }
}
