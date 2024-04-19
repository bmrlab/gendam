use async_trait::async_trait;
use std::{fmt::Display, hash::Hash};

#[async_trait]
pub trait FileHandler<TTaskType, TMetadata>: Clone
where
    TTaskType: Clone + Display + PartialEq + Eq + Hash,
    TMetadata: Clone,
{
    async fn run_task(&self, task_type: &TTaskType) -> anyhow::Result<()>;
    async fn delete_task_artifacts(&self, task_type: &TTaskType) -> anyhow::Result<()>;
    async fn update_database(&self) -> anyhow::Result<()>;

    fn get_supported_task_types(&self) -> Vec<TTaskType>;
    fn metadata(&self) -> anyhow::Result<TMetadata>;
}
