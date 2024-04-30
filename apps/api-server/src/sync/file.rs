use std::{collections::HashSet, path::Path};

use content_library::Library;
use p2p::PubsubMessage;
use prisma_lib::file_path;
use sync_lib::SyncError;

use autosurgeon::{hydrate, reconcile};

use crate::{
    sync::{File, Folder, Item},
    utils::path::{
        get_folder_info_by_path, get_folder_list, get_name_and_materialized_path_by_full_path,
        get_suffix_path,
    },
};

/*
    修改
    1. 改名
    2. 更新自己的文档
    3. 更新包含自己的文件夹文档
*/
pub async fn update_file_crdt(
    library: &Library,
    old_path: String,
    new_name: Option<String>,
) -> Result<(), SyncError> {
    if new_name.is_none() {
        return Ok(());
    }
    // todo 不应该放在这里，不能广播，但应该改crdt
    if let Some(broadcast) = library.get_broadcast() {
        let sync = library.sync();
        // 先查询自己的文档
        let path = Path::new(&old_path);
        let path_split = path.components().collect::<Vec<_>>();
        let old_name = path_split
            .last()
            .unwrap()
            .as_os_str()
            .to_str()
            .unwrap()
            .to_string();
        // 取第一个到倒数第二个
        let mut materialized_path = path_split[0].as_os_str().to_str().unwrap().to_string();
        for i in 1..path_split.len() - 1 {
            materialized_path.push_str("/");
            materialized_path.push_str(path_split[i].as_os_str().to_str().unwrap());
        }
        let new_name_str = new_name.clone().unwrap();
        // 查询filepath
        let file_path = library
            .prisma_client()
            .file_path()
            .find_unique(file_path::UniqueWhereParam::MaterializedPathNameEquals(
                materialized_path.clone(),
                old_name.clone(),
            ))
            .exec()
            .await
            .unwrap()
            .unwrap(); // 一定有

        if let Some(doc_id) = file_path.doc_id {
            // 说明是自己的文档
            let document_id: automerge_repo::DocumentId = doc_id.parse().expect("parse error");
            // 加载文档
            let dochandle = sync
                .request_document(document_id)
                .await
                .expect("load doc error");

            dochandle.with_doc_mut(|doc| {
                // 修改文档
                let mut sync_item: File = hydrate(doc).expect("hydrate error");
                // name
                sync_item.name = new_name.unwrap_or(sync_item.name);
                let mut tx = doc.transaction();
                reconcile(&mut tx, &sync_item).unwrap();
                // 保存文档
                tx.commit();
            });
            let _ = broadcast.send(PubsubMessage::Sync(doc_id)).await;
        }
        // 查找包含这个文件的文件夹文档
        // 找到所有文件夹
        let mut folder_list = Vec::new();
        // 除了第一个 /
        for i in 1..path_split.len() - 1 {
            let mut folder_path = "".to_string();
            for j in 1..=i {
                folder_path.push_str("/");
                folder_path.push_str(path_split[j].as_os_str().to_str().unwrap());
            }
            folder_list.push(folder_path);
        }
        tracing::debug!("folder_list:{folder_list:#?}");
        for folder_path_string in folder_list {
            let (_folder_path, _folder_materialized_path, _folder_name, folder_data) =
                get_folder_info_by_path(library.prisma_client(), folder_path_string.clone())
                    .await
                    .unwrap();

            if let Some(folder_doc_id) = folder_data.doc_id {
                // 说明有同步文件夹文档
                // 加载文档
                let folder_document_id: automerge_repo::DocumentId =
                    folder_doc_id.clone().parse().expect("parse error");
                let folder_dochandle = sync.request_document(folder_document_id).await.unwrap();
                folder_dochandle.with_doc_mut(|doc| {
                    // 修改文档
                    let mut folder_value: Folder = hydrate(doc).expect("hydrate error");
                    // 修改文件
                    let mut tx = doc.transaction();
                    for item in &mut folder_value.children {
                        if item.path == old_path {
                            tracing::debug!("materialized_path: {materialized_path:?}, new_name_str:{new_name_str:?}");
                            item.path =
                                format!("{}{}", materialized_path.clone(), new_name_str.clone());
                            break;
                        }
                    }
                    tracing::debug!("folder_value:{:#?}", folder_value);
                    reconcile(&mut tx, &folder_value).unwrap();
                    tx.commit();
                });
                let _ = broadcast.send(PubsubMessage::Sync(folder_doc_id)).await;
            }
        }
    }
    Ok(())
}

/*
    移动文件
    从数据库中加载文件所属的文档
    专属于自己的文档不用管，其他的文件夹文档需要更新这个文件相对路径
*/
pub async fn move_file_crdt(
    library: &Library,
    old_path: String,
    new_path: Option<String>,
) -> Result<(), SyncError> {
    // 移动完了
    if new_path.is_none() {
        return Ok(());
    }

    // 先查询这个文件
    let (name, materialized_path) = get_name_and_materialized_path_by_full_path(old_path.clone());
    let data = library
        .prisma_client()
        .file_path()
        .find_unique(file_path::UniqueWhereParam::MaterializedPathNameEquals(
            materialized_path,
            name,
        ))
        .with(file_path::asset_object::fetch())
        .exec()
        .await?
        .unwrap();
    let doc_id = data.doc_id;
    let hash = Some(data.asset_object.unwrap().unwrap().hash);

    if let Some(broadcast) = library.get_broadcast() {
        if let Some(new_path) = new_path {
            tracing::debug!("move_file_crdt: old_path:{old_path:?}, new_path:{new_path:?}");
            let sync = library.sync();
            let mut hash_set = HashSet::new();
            // 这是包含文件的文件夹路径
            let folder_list = get_folder_list(old_path.clone(), true);
            let new_folder_list = get_folder_list(new_path.clone(), true);

            for folder in folder_list.clone() {
                hash_set.insert(folder);
            }
            for folder in new_folder_list.clone() {
                hash_set.insert(folder);
            }

            let folder_list: Vec<String> = hash_set.into_iter().collect();

            tracing::debug!("folder_list: {folder_list:?}");

            for folder_path_string in folder_list {
                let (_folder_path, folder_materialized_path, folder_name, folder_data) =
                    get_folder_info_by_path(library.prisma_client(), folder_path_string.clone())
                        .await
                        .unwrap();

                if let Some(folder_doc_id) = folder_data.doc_id {
                    // 说明有同步文件夹文档
                    // 加载文档
                    let folder_document_id: automerge_repo::DocumentId =
                        folder_doc_id.clone().parse().expect("parse error");
                    let folder_dochandle = sync.request_document(folder_document_id).await.unwrap();

                    folder_dochandle.with_doc_mut(|doc| {
                        // 修改文档
                        let mut folder_value: Folder = hydrate(doc).expect("hydrate error");
                        // 修改文件
                        let mut tx = doc.transaction();
                        // 分2种情况
                        // 1. 移动到共享文件夹下的其他目录，需要更新这个文件的相对路径
                        // 2. 移动到不属于共享文件夹的目录， 需要删除这个文件
                        // 共享文件夹的路径
                        let folder_path_string =
                            format!("{}{}/", folder_materialized_path, folder_name);
                        tracing::debug!("new_path:{new_path:?}");
                        tracing::debug!("folder_path_string:{folder_path_string:?}");

                        // 共享文件夹的路径是否是 文件移动的新路径的前缀
                        if new_path.starts_with(&folder_path_string) {
                            // 移动到共享文件夹下的其他目录
                            let new_relative_path = new_path
                                .strip_prefix(&folder_path_string)
                                .unwrap()
                                .to_string();

                            match old_path.strip_prefix(&folder_path_string) {
                                Some(old_relative_path) => {
                                    tracing::debug!("old_relative_path:{old_relative_path:?}");
                                    tracing::debug!("new_relative_path: {new_relative_path:?}");

                                    for item in &mut folder_value.children {
                                        if item.path == old_relative_path {
                                            item.path = new_relative_path.to_string();
                                            break;
                                        }
                                    }
                                }
                                None => {
                                    // 说明从其他地方移动进去的
                                    // 新增这个
                                    folder_value.push(Item {
                                        path: new_relative_path.to_string(),
                                        doc_id: doc_id.clone(),
                                        is_dir: false,
                                        hash: hash.clone(),
                                    })
                                }
                            }
                        } else {
                            // 移动到不属于共享文件夹的目录
                            for item in &mut folder_value.children {
                                if item.path == old_path {
                                    folder_value.children.retain(|x| x.path != old_path);
                                    break;
                                }
                            }
                        }
                        folder_value.sort();
                        tracing::debug!("移动文件之后的 folder_value:{:#?}", folder_value);
                        reconcile(&mut tx, &folder_value).unwrap();
                        tx.commit();
                    });
                    let _ = broadcast.send(PubsubMessage::Sync(folder_doc_id)).await;
                }
            }
        }
    }
    Ok(())
}

/*
    todo 删除文件
    1. 自己的文档 要不要保留
    2. 属于其他文件夹文档需要更新删除这个文件
*/
pub async fn delete_crdt(
    library: &Library,
    materialized_path: String,
    name: String,
) -> Result<(), SyncError> {
    let sync = library.sync();
    // 先查询数据
    let file_path = library
        .prisma_client()
        .file_path()
        .find_unique(file_path::UniqueWhereParam::MaterializedPathNameEquals(
            materialized_path.clone(),
            name.clone(),
        ))
        .exec()
        .await
        .unwrap()
        .unwrap(); // 一定有

    let is_dir = file_path.is_dir;

    if let Some(doc_id) = file_path.doc_id {
        // 自己的文档删除了
        sync.delete_document(doc_id.clone()).await.unwrap();
    }
    // 如果是文件夹，还要删除这个文件夹下的所有子项的文档
    if is_dir {
        tracing::debug!("delete_crdt materialized_path:{materialized_path:?}, name:{name:?}");
        let data: Vec<_> = library
            .prisma_client()
            .file_path()
            .find_many(vec![file_path::materialized_path::starts_with(format!(
                "{}{}/",
                materialized_path, name
            ))])
            .exec()
            .await
            .unwrap();

        tracing::debug!("delete_crdt data:{data:?}");
        // todo 研究一下
        // for item in data {
        //     if let Some(doc_id) = item.doc_id {
        //         sync.delete_document(doc_id).await.unwrap();
        //     }
        // }
    }

    let mut folder_list = Vec::new();

    // 查找包含这个文件或者文件夹的文件夹文档
    let folder_path_string = format!("{}{}", materialized_path, name);
    let folder_path = Path::new(folder_path_string.as_str());
    let folder_path_split = folder_path.components().collect::<Vec<_>>();
    // 取第一个到倒数第二个

    for i in 1..folder_path_split.len() - 1 {
        let mut folder_path = "".to_string();
        for j in 1..=i {
            folder_path.push_str("/");
            folder_path.push_str(folder_path_split[j].as_os_str().to_str().unwrap());
        }
        folder_list.push(folder_path);
    }

    tracing::debug!("folder_list: {:?}", folder_list);

    for folder_path_string in folder_list {
        let (_folder_path, folder_materialized_path, folder_name, folder_data) =
            get_folder_info_by_path(library.prisma_client(), folder_path_string.clone())
                .await
                .unwrap();

        if let Some(doc_id) = folder_data.doc_id {
            let document_id: automerge_repo::DocumentId = doc_id.parse().expect("parse error");
            let dochandle = sync
                .request_document(document_id)
                .await
                .expect("load doc error");

            let relation_path = get_suffix_path(
                &format!("{}{}", materialized_path, name),
                &format!("{}{}", folder_materialized_path, folder_name),
            );

            tracing::debug!("relation_path:{relation_path:?}");

            dochandle.with_doc_mut(|doc| {
                let mut folder: Folder = hydrate(doc).expect("hydrate error");
                folder.remove(relation_path, is_dir);
                tracing::debug!("folder:{:#?}", folder);
                let mut tx = doc.transaction();
                reconcile(&mut tx, &folder).unwrap();
                tx.commit();
            });
            let _ = library
                .get_broadcast()
                .unwrap()
                .send(PubsubMessage::Sync(doc_id))
                .await;
        }
    }

    // todo 对方的文档，如果没有父级文件夹，就不管这个文件夹
    // 如果有父级文件夹，就删除这个文件夹下的这个文件
    Ok(())
}
