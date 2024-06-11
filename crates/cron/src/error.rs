use thiserror::Error;

#[derive(Error, Debug)]
pub enum CronError {
    #[error("cron error: {0}")]
    Cron(#[from] async_cron_scheduler::cron::error::Error),

    #[error("task {0} no found")]
    TaskNotFound(uuid::Uuid),

    #[error("scheduler not running")]
    SchedulerNotRunning,

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("invalid format error")]
    InvalidFormat,

    #[error("start error")]
    Start
}
