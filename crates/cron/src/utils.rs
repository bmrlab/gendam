use std::{pin::Pin, str::FromStr, sync::Arc};

use async_cron_scheduler::cron;
use futures::{future::BoxFuture, Future};

use crate::error::CronError;

pub fn parse_cron(s: &str) -> Result<cron::Schedule, CronError> {
    match cron::Schedule::from_str(s) {
        Ok(schedule) => Ok(schedule),
        Err(e) => Err(CronError::Cron(e)),
    }
}

/// create job fn
///
/// ```
/// cron::create_job_fn(|| async { println!("123") }.boxed())
/// ```
pub fn create_job_fn<F>(job_fn: F) -> Arc<dyn Fn() -> BoxFuture<'static, ()> + Send + Sync>
where
    F: Fn() -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync + 'static,
{
    Arc::new(job_fn)
}
