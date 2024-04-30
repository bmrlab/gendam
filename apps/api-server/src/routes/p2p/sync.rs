use automerge_repo::DocumentId;
use autosurgeon::reconcile;
use prisma_lib::{file_path, PrismaClient};
use std::{str::FromStr, sync::Arc};
use sync_lib::Sync;

use crate::sync::{File, Folder};

use super::info::{DocIdWithFolder, DocIdWithHash};

// 分享的时候初始化文档
pub async fn init_sync(
    prisma_client: Arc<PrismaClient>,
    sync: Sync,
    file_path_id_list: Vec<i32>,
) -> Result<(Vec<DocIdWithHash>, Vec<DocIdWithFolder>), anyhow::Error> {
    tracing::debug!("start init sync: {file_path_id_list:?}");

    let file_paths = prisma_client
        .file_path()
        .find_many(vec![file_path::id::in_vec(file_path_id_list.clone())])
        .with(file_path::asset_object::fetch())
        .exec()
        .await
        .unwrap();
    tracing::debug!("file_paths: {file_paths:#?}");

    let mut doc_id_list: Vec<DocIdWithHash> = Vec::new();
    let mut folder_list: Vec<DocIdWithFolder> = Vec::new();
    for file_path in file_paths {
        // 先判断是不是文件夹
        match file_path.is_dir {
            true => {
                // 是文件夹
                let folder = Folder::from_db(
                    prisma_client.clone(),
                    file_path.clone().materialized_path,
                    file_path.clone().name,
                )
                .await
                .unwrap();

                // 查找文件夹下所有文件夹和文件
                let root_path = file_path.materialized_path.clone() + &file_path.name.clone() + "/";
                let sub_file_res = prisma_client
                    .file_path()
                    .find_many(vec![file_path::materialized_path::starts_with(
                        root_path.clone(),
                    )])
                    .with(file_path::asset_object::fetch())
                    .exec()
                    .await
                    .unwrap();

                for sub_file in sub_file_res {
                    if !sub_file.is_dir {
                        let _ = init_file(
                            sub_file,
                            &mut doc_id_list,
                            prisma_client.clone(),
                            sync.clone(),
                        )
                        .await?;
                    };
                }

                tracing::debug!("folder: {folder:#?}");
                let folder_doc_handle = match file_path.doc_id {
                    Some(doc_id) => {
                        let document_id =
                            DocumentId::from_str(&doc_id.clone()).expect("fail parse doc_id");
                        let doc = sync
                            .request_document(document_id)
                            .await
                            .expect("fail request document");

                        doc.with_doc_mut(|doc| {
                            let mut tx = doc.transaction();
                            reconcile(&mut tx, &folder).unwrap();
                            tx.commit();
                        });
                        doc
                    }
                    None => {
                        let new_doc = sync.new_document();
                        let doc_id = new_doc.document_id();
                        let doc_id_string = doc_id.as_uuid_str();

                        new_doc.with_doc_mut(|doc| {
                            let mut tx = doc.transaction();
                            reconcile(&mut tx, &folder).unwrap();
                            tx.commit();
                        });
                        // 建立关系
                        prisma_client
                            .file_path()
                            .update(
                                file_path::UniqueWhereParam::IdEquals(file_path.id),
                                vec![file_path::doc_id::set(Some(doc_id_string.clone()))],
                            )
                            .exec()
                            .await
                            .expect("fail update file path");
                        new_doc
                    }
                };

                let folder_doc_id_string = folder_doc_handle.document_id().as_uuid_str();

                folder_list.push(DocIdWithFolder {
                    doc_id: folder_doc_id_string,
                    folder,
                });
            }
            false => {
                // 是文件
                init_file(
                    file_path,
                    &mut doc_id_list,
                    prisma_client.clone(),
                    sync.clone(),
                )
                .await?;
            }
        }
    }

    tracing::debug!("doc_id_list: {doc_id_list:#?}");
    tracing::debug!("folder_list: {folder_list:#?}");
    Ok((doc_id_list, folder_list))
}

pub async fn init_file(
    file_path: file_path::Data,
    doc_id_list: &mut Vec<DocIdWithHash>,
    prisma_client: Arc<PrismaClient>,
    sync: Sync,
) -> Result<(), anyhow::Error> {
    match file_path.doc_id {
        Some(doc_id) => doc_id_list.push(DocIdWithHash {
            hash: file_path.asset_object.unwrap().unwrap().hash,
            name: file_path.name,
            doc_id,
        }),
        None => {
            let new_doc = sync.new_document();
            let doc_id = new_doc.document_id();
            let doc_id_string = doc_id.as_uuid_str();
            // 生成 syncitem
            let sync_item = File {
                name: file_path.name.clone(),
                hash: file_path.asset_object.clone().unwrap().unwrap().hash,
            };

            new_doc.with_doc_mut(|doc| {
                let mut tx = doc.transaction();
                reconcile(&mut tx, &sync_item).unwrap();
                tx.commit();
            });

            // 更新 file_path 和 文档关系
            prisma_client
                .file_path()
                .update(
                    file_path::UniqueWhereParam::IdEquals(file_path.id),
                    vec![file_path::doc_id::set(Some(doc_id_string.clone()))],
                )
                .exec()
                .await
                .expect("fail update file path");

            doc_id_list.push(DocIdWithHash {
                hash: file_path.asset_object.unwrap().unwrap().hash,
                name: file_path.name,
                doc_id: doc_id_string,
            });
        }
    };
    Ok(())
}
