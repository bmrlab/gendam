use std::sync::Arc;

use automerge_repo::{DocumentId, Storage as RepoStorageTrait, StorageError};
use futures::future::BoxFuture;
use futures::{FutureExt, TryFutureExt};
use prisma_lib::PrismaClient;
use tokio::sync::mpsc::{channel, Sender};
use tokio::sync::oneshot::{self, channel as oneshotChannel};

use crate::utils::{append, compact, list_all, load};

#[derive(Debug)]
enum StorageRequest {
    Load(DocumentId, oneshot::Sender<Option<Vec<u8>>>),
    Append(DocumentId, Vec<u8>, oneshot::Sender<()>),
    Compact(DocumentId, Vec<u8>, oneshot::Sender<()>),
    ListAll(oneshot::Sender<Vec<DocumentId>>),
}

pub struct Storage {
    chan: Sender<StorageRequest>,
}

impl Storage {
    pub fn new(db: Arc<PrismaClient>) -> Self {
        let (tx, mut rx) = channel(1);
        tokio::spawn(async move {
            while let Some(request) = rx.recv().await {
                match request {
                    StorageRequest::Load(id, reply) => {
                        tracing::debug!("sync storage load document {:?}", id.as_uuid_str());
                        if let Ok(result) = load(db.clone(), id).await {
                            reply.send(result).unwrap();
                        }
                    }
                    StorageRequest::Append(id, changes, reply) => {
                        tracing::debug!("sync storage append document {:?}", id.as_uuid_str());
                        if let Ok(()) = append(db.clone(), id, changes).await {
                            reply.send(()).unwrap();
                        }
                    }
                    StorageRequest::Compact(id, full_doc, reply) => {
                        tracing::debug!("sync storage compact document {:?}", id.as_uuid_str());
                        if let Ok(()) = compact(db.clone(), id, full_doc).await {
                            reply.send(()).unwrap();
                        }
                    }
                    StorageRequest::ListAll(reply) => {
                        tracing::debug!("sync storage list all documents");
                        if let Ok(result) = list_all(db.clone()).await {
                            reply.send(result).unwrap();
                        }
                    }
                }
            }
        });

        Self { chan: tx }
    }
}

impl RepoStorageTrait for Storage {
    fn get(&self, id: DocumentId) -> BoxFuture<'static, Result<Option<Vec<u8>>, StorageError>> {
        let (tx, rx) = oneshotChannel();
        let request = StorageRequest::Load(id, tx);
        self.chan.blocking_send(request).unwrap();
        rx.map_err(|_| StorageError::Error).boxed()
    }

    fn list_all(&self) -> BoxFuture<'static, Result<Vec<DocumentId>, StorageError>> {
        let (tx, rx) = oneshotChannel();
        self.chan
            .blocking_send(StorageRequest::ListAll(tx))
            .unwrap();
        rx.map_err(|_| StorageError::Error).boxed()
    }

    fn append(
        &self,
        id: DocumentId,
        changes: Vec<u8>,
    ) -> BoxFuture<'static, Result<(), StorageError>> {
        let (tx, rx) = oneshotChannel();
        self.chan
            .blocking_send(StorageRequest::Append(id, changes, tx))
            .unwrap();
        rx.map_err(|_| StorageError::Error).boxed()
    }

    fn compact(
        &self,
        id: DocumentId,
        full_doc: Vec<u8>,
    ) -> BoxFuture<'static, Result<(), StorageError>> {
        let (tx, rx) = oneshotChannel();
        self.chan
            .blocking_send(StorageRequest::Compact(id, full_doc, tx))
            .unwrap();
        rx.map_err(|_| StorageError::Error).boxed()
    }
}
