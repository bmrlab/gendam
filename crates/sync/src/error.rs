use std::fmt;
use automerge::AutomergeError;
use automerge_repo::{NetworkError, RepoError};
use autosurgeon::HydrateError;
use autosurgeon::ReconcileError;
use prisma_client_rust::QueryError;
use thiserror::Error;

// 封装 RepoError 的新类型
#[derive(Debug)]
pub struct WrappedRepoError(pub RepoError);

impl fmt::Display for WrappedRepoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Repo error: {:?}", self.0)
    }
}

impl std::error::Error for WrappedRepoError {}

#[derive(Error, Debug)]
pub enum SyncError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Automerge error: {0}")]
    Automerge(#[from] AutomergeError),

    #[error("Prisma error: {0}")]
    Prisma(#[from] QueryError),

    #[error("Hydrate error: {0}")]
    Hydrate(#[from] HydrateError),

    #[error("Reconcile error: {0}")]
    Reconcile(#[from] ReconcileError),

    #[error("Network error: {0}")]
    Network(#[from] NetworkError),

    #[error("Repo error: {0}")]
    Repo(#[from] WrappedRepoError),

    #[error("Invalid UUid error: {0}")]
    InvalidUUid(String),

    #[error("Other error")]
    Other,
}

// 从 RepoError 创建 WrappedRepoError 的转换
impl From<RepoError> for WrappedRepoError {
    fn from(err: RepoError) -> WrappedRepoError {
        WrappedRepoError(err)
    }
}