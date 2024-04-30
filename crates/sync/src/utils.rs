use std::{str::FromStr, sync::Arc};

use automerge::sync::ReadMessageError;
use automerge_repo::DocumentId;
use prisma_lib::{sync, PrismaClient};

use crate::SyncError;

pub fn encode_sync_message(message: automerge::sync::Message) -> Vec<u8> {
    message.encode()
}

pub fn decode_sync_message(bytes: &[u8]) -> Result<automerge::sync::Message, ReadMessageError> {
    automerge::sync::Message::decode(bytes)
}

/*
    storage 相关的操作
*/
pub async fn load(
    prisma_client: Arc<PrismaClient>,
    doc_id: DocumentId,
) -> Result<Option<Vec<u8>>, SyncError> {
    let doc_id_str = doc_id.as_uuid_str();

    match prisma_client
        .sync()
        .find_unique(sync::doc_id::equals(doc_id_str))
        .exec()
        .await?
    {
        Some(data) => Ok(Some(data.doc)),
        None => Ok(None),
    }
}

pub async fn append(
    prisma_client: Arc<PrismaClient>,
    doc_id: DocumentId,
    changes: Vec<u8>,
) -> Result<(), SyncError> {
    let doc_id_str = doc_id.as_uuid_str();

    match prisma_client
        .sync()
        .find_unique(sync::doc_id::equals(doc_id_str.clone()))
        .exec()
        .await?
    {
        Some(data) => {
            let mut new_doc = data.doc.clone();
            new_doc.extend_from_slice(&changes);
            let _ = prisma_client
                .sync()
                .update(sync::id::equals(data.id), vec![sync::doc::set(new_doc)])
                .exec()
                .await?;
        }
        None => {
            // 说明第一次创建
            let _ = prisma_client
                .sync()
                .create(doc_id_str, changes, vec![])
                .exec()
                .await?;
        }
    }
    Ok(())
}

pub async fn compact(
    prisma_client: Arc<PrismaClient>,
    doc_id: DocumentId,
    full_doc: Vec<u8>,
) -> Result<(), SyncError> {
    let doc_id_str = doc_id.as_uuid_str();

    match prisma_client
        .sync()
        .find_unique(sync::doc_id::equals(doc_id_str.clone()))
        .exec()
        .await?
    {
        Some(data) => {
            // 更新数据
            let _ = prisma_client
                .sync()
                .update(sync::id::equals(data.id), vec![sync::doc::set(full_doc)])
                .exec()
                .await?;
        }
        None => {
            // create
            let _ = prisma_client
                .sync()
                .create(doc_id_str, full_doc, vec![])
                .exec()
                .await?;
        }
    }
    Ok(())
}

pub async fn list_all(prisma_client: Arc<PrismaClient>) -> Result<Vec<DocumentId>, SyncError> {
    let data = prisma_client.sync().find_many(vec![]).exec().await?;
    Ok(data
        .into_iter()
        .map(|d| DocumentId::from_str(&d.doc_id).expect("invalid document ID"))
        .collect::<Vec<_>>())
}

pub fn str_to_document_id(id: String) -> Result<DocumentId, SyncError> {
    Ok(DocumentId::from_str(&id).map_err(|_error| SyncError::InvalidUUid(id))?)
}

pub(crate) async fn delete_document_by_id_string(
    prisma_client: Arc<PrismaClient>,
    doc_id: String,
) -> Result<(), SyncError> {
    let _ = prisma_client
        .sync()
        .delete(sync::UniqueWhereParam::DocIdEquals(doc_id))
        .exec()
        .await?;
    Ok(())
}

pub(crate) async fn get_document_by_id(
    prisma_client: Arc<PrismaClient>,
    doc_id: DocumentId,
) -> Result<bool, SyncError> {
    let doc_id_string = doc_id.as_uuid_str();
    let has_doc = prisma_client
        .sync()
        .find_unique(sync::UniqueWhereParam::DocIdEquals(doc_id_string))
        .exec()
        .await?;
    Ok(has_doc.is_some())
}
