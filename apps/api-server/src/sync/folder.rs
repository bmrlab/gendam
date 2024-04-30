use std::path::Path;

use autosurgeon::{hydrate, reconcile};
use content_library::Library;
use p2p::PubsubMessage;
use prisma_lib::file_path;
use sync_lib::SyncError;

use crate::{
    sync::{File, Folder, Item},
    utils::path::{
        get_folder_info_by_path, get_folder_list, get_new_path_by_old_path_and_name,
        get_suffix_path,
    },
};

/*
    修改
    1. 改名
    2. 修改描述
*/
pub async fn update_folder_crdt(
    library: &Library,
    old_path: String,
    new_name: Option<String>,
) -> Result<(), SyncError> {
    tracing::debug!("old_path: {old_path:?}, new_name:{new_name:?}");
    let sync = library.sync();
    if new_name.is_none() {
        return Ok(());
    }
    let folder_list = get_folder_list(old_path.clone(), false);

    let new_path_string =
        get_new_path_by_old_path_and_name(old_path.clone(), new_name.clone().unwrap());

    tracing::debug!("new_path_string: {new_path_string:?}");

    for folder_path_string in folder_list {
        let (folder_path, _folder_materialized_path, _folder_name, folder_data) =
            get_folder_info_by_path(library.prisma_client(), folder_path_string.clone())
                .await
                .unwrap();

        if let Some(doc_id) = folder_data.doc_id {
            let document_id: automerge_repo::DocumentId = doc_id.parse().expect("parse error");
            let dochandle = sync
                .request_document(document_id)
                .await
                .expect("load doc error");

            dochandle.with_doc_mut(|doc| {
                let mut folder: Folder = hydrate(doc).expect("hydrate error");

                if folder_path_string == old_path.clone() {
                    // 如果是自己的文件夹文档
                    // 就修改名字
                    folder.name = new_name.clone().unwrap_or(folder.name);
                    // 下面的children的path不用管，因为是相对路径，不包含name
                } else {
                    // 如果是其他文件夹文档
                    // 需要找到自己，更新相对路径，再找到自己下面的所有子项，更新相对路径
                    for item in &mut folder.children {
                        // 通过folder_path 和 old_path 找到自己的相对路径
                        let old_relative_path =
                            get_suffix_path(&old_path, folder_path.to_str().unwrap());
                        tracing::debug!("old_relative_path: {old_relative_path:?}");
                        // 更新自己的相对路径
                        if item.path == old_relative_path {
                            item.path =
                                get_suffix_path(&new_path_string, folder_path.to_str().unwrap());
                        }

                        // 更新自己下面的所有子项的相对路径
                        if item.path.starts_with(&old_relative_path) {
                            tracing::debug!("子项 {item:?}");
                            // path前面去掉old_relative_path
                            let suffix = get_suffix_path(&item.path, &old_relative_path);
                            item.path = Path::new(&new_path_string)
                                .strip_prefix(&folder_path)
                                .unwrap()
                                .join(Path::new(&suffix))
                                .to_str()
                                .unwrap()
                                .to_string();
                        }
                    }
                }
                tracing::debug!("修改名称后的 folder: {:#?}", folder);
                let mut tx = doc.transaction();
                reconcile(&mut tx, &folder).unwrap();
                tx.commit();
            });
            if let Some(broadcast) = library.get_broadcast() {
                let _ = broadcast.send(PubsubMessage::Sync(doc_id)).await;
            }
        }
    }

    Ok(())
}

/*
    移动文件夹
    1. 属于自己的文件夹文档不用管
    2. 属于其他的文件夹文档需要更新相对路径
*/

pub async fn move_folder_crdt(
    library: &Library,
    old_path: String,
    new_path: Option<String>,
) -> Result<(), SyncError> {
    if new_path.is_none() {
        return Ok(());
    };
    if let Some(new_path) = new_path {
        tracing::debug!("move_folder_crdt: old_path:{old_path:?}, new_path:{new_path:?}");
        // 这是包含文件的文件夹路径

        let folder_list = get_folder_list(old_path.clone(), true);

        // ["/111"]
        for folder_path_string in folder_list {
            let (folder_path, folder_materialized_path, folder_name, folder_data) =
                get_folder_info_by_path(library.prisma_client(), folder_path_string.clone())
                    .await
                    .unwrap();

            if let Some(doc_id) = folder_data.doc_id {
                let document_id: automerge_repo::DocumentId = doc_id.parse().expect("parse error");
                let sync = library.sync();
                let dochandle = sync
                    .request_document(document_id)
                    .await
                    .expect("load doc error");

                // 重新生成一次folder
                // 还不能生成，这个crdt在移动后面

                dochandle.with_doc_mut(|doc| {
                    let mut folder: Folder = hydrate(doc).expect("hydrate error");
                    // 分两种情况
                    // 1. 移动到 共享文件夹下的其他路径，需要更新这个文件夹相对路径和这个文件夹下的所有子项的相对路径
                    // 2. 移动到共享文件夹外，需要删除这个文件夹，和这个文件夹下的所有子项
                    let folder_path_string = format!("{}{}", folder_materialized_path, folder_name);

                    if new_path.starts_with(&folder_path_string) {
                        tracing::debug!("new_path: {new_path:?}");
                        tracing::debug!("folder_path_string: {folder_path_string:?}");
                        // 需要更新这个文件夹相对路径和这个文件夹下的所有子项的相对路径
                        for item in &mut folder.children {
                            // 通过folder_path 和 old_path 找到自己的相对路径
                            let old_relative_path =
                                get_suffix_path(&old_path, folder_path.to_str().unwrap());

                            tracing::debug!("old_relative_path: {old_relative_path:?}");

                            // 更新自己的相对路径
                            if item.path == old_relative_path {
                                let new_path =
                                    get_suffix_path(&new_path, folder_path.to_str().unwrap());
                                tracing::debug!("new_path: {new_path:?}");
                                item.path = new_path
                            }
                            // 更新自己下面的所有子项的相对路径
                            if item.path.starts_with(&old_relative_path) {
                                // item.path 去掉 old_relative_path
                                let item_relation_path =
                                    get_suffix_path(&item.path, &old_relative_path);

                                let children_new_path = Path::new(&new_path)
                                    .strip_prefix(&folder_path)
                                    .unwrap()
                                    .join(Path::new(&item_relation_path)) // 这里join应该是 item和被移动文件夹的相对位置
                                    .to_str()
                                    .unwrap()
                                    .to_string();

                                tracing::debug!("children_new_path: {children_new_path:?}");
                                item.path = children_new_path
                            }
                        }
                    } else {
                        // 需要删除这个文件夹，和这个文件夹下的所有子项
                        folder.children.retain(|item| {
                            !item.path.starts_with(
                                Path::new(&old_path)
                                    .strip_prefix(&folder_path)
                                    .unwrap()
                                    .to_str()
                                    .unwrap(),
                            )
                        });
                    }
                    folder.sort();
                    tracing::debug!("移动后的 folder: {:#?}", folder);
                    let mut tx = doc.transaction();
                    reconcile(&mut tx, &folder).unwrap();
                    tx.commit();
                });
                if let Some(broadcast) = library.get_broadcast() {
                    let _ = broadcast.send(PubsubMessage::Sync(doc_id)).await;
                }
            }
        }
    }

    Ok(())
}

/*
    todo 创建文件夹
    1. 创建文件夹
    2. 创建文件夹文档
    3. 更新父文件夹文档
*/

pub async fn create_folder_crdt(
    library: &Library,
    materialized_path: String,
    name: String,
) -> Result<(), SyncError> {
    let sync = library.sync();
    let folder_list = get_folder_list(format!("{}{}", materialized_path, name), true);
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

            dochandle.with_doc_mut(|doc| {
                let mut folder: Folder = hydrate(doc).expect("hydrate error");
                // 相对路径
                let relative_path = get_suffix_path(
                    &format!("{}{}", materialized_path, name),
                    &format!("{}{}", folder_materialized_path, folder_name),
                );

                folder.children.push(Item {
                    path: relative_path,
                    is_dir: true,
                    doc_id: None,
                    hash: None,
                });
                folder.sort();
                tracing::debug!("创建文件夹之后的 folder_value:{:#?}", folder);
                let mut tx = doc.transaction();
                reconcile(&mut tx, &folder).unwrap();
                tx.commit();
            });
            if let Some(broadcast) = library.get_broadcast() {
                let _ = broadcast.send(PubsubMessage::Sync(doc_id)).await;
            }
        }
    }

    Ok(())
}

// 文件任务完成，触发这个函数
pub async fn update_doc_add_new_file(
    library: &Library,
    path: String,
    name: String,
) -> Result<Vec<String>, SyncError> {
    tracing::debug!("update_doc_add_new_file: {path:?} {name:?}");
    let sync = library.sync();
    // 查询hash
    let file_data = library
        .prisma_client()
        .file_path()
        .find_unique(file_path::UniqueWhereParam::MaterializedPathNameEquals(
            path.clone(),
            name.clone(),
        ))
        .with(file_path::asset_object::fetch())
        .exec()
        .await
        .unwrap()
        .unwrap();

    let hash = file_data.asset_object.unwrap().unwrap().hash; //todo 优化

    // 这是父文件夹文档id list
    let mut doc_id_list = Vec::new();
    let file_path = Path::new(&path);
    let path_split = file_path.components().collect::<Vec<_>>();
    // 这是包含文件的文件夹路径
    let mut folder_list = Vec::new();

    for i in 1..path_split.len() {
        let mut folder_path = "".to_string();
        for j in 1..=i {
            folder_path.push_str("/");
            folder_path.push_str(path_split[j].as_os_str().to_str().unwrap());
        }
        folder_list.push(folder_path);
    }

    // 新文件生成文档
    let new_doc = sync.new_document();
    let doc_id = new_doc.document_id();
    let doc_id_string = doc_id.as_uuid_str();

    // 生成 syncitem
    let sync_item = File {
        name: name.clone(),
        hash: hash.clone(),
    };

    new_doc.with_doc_mut(|doc| {
        let mut tx = doc.transaction();
        reconcile(&mut tx, &sync_item).unwrap();
        tx.commit();
    });

    // 更新
    let _ = library
        .prisma_client()
        .file_path()
        .update(
            file_path::UniqueWhereParam::MaterializedPathNameEquals(path.clone(), name.clone()),
            vec![file_path::doc_id::set(Some(doc_id_string.clone()))],
        )
        .exec()
        .await
        .expect("fail update file path");

    tracing::debug!("folder_list: {folder_list:?}");

    for folder_path_string in folder_list {
        let (_folder_path, folder_materialized_path, folder_name, folder_data) =
            get_folder_info_by_path(library.prisma_client(), folder_path_string.clone())
                .await
                .unwrap();

        if let Some(doc_id) = folder_data.doc_id {
            doc_id_list.push(doc_id.clone());
            let document_id: automerge_repo::DocumentId = doc_id.parse().expect("parse error");
            let sync = library.sync();
            let dochandle = sync
                .request_document(document_id)
                .await
                .expect("load doc error");

            let folder_path_string = format!("{}{}", folder_materialized_path, folder_name);

            let relation_path_string = get_suffix_path(
                &format!("{}{}", path.clone(), name.clone()),
                &folder_path_string,
            );

            dochandle.with_doc_mut(|doc| {
                let mut folder: Folder = hydrate(doc).expect("hydrate error");
                folder.children.push(Item {
                    path: relation_path_string,
                    is_dir: false,
                    doc_id: Some(doc_id_string.clone()),
                    hash: Some(hash.clone()),
                });
                folder.sort();
                let mut tx = doc.transaction();
                reconcile(&mut tx, &folder).unwrap();
                tx.commit();
            });
        }
    }

    tracing::debug!("doc_id_list: {doc_id_list:?}");
    // 再触发一次同步
    if let Some(broadcast) = library.get_broadcast() {
        for doc_id in doc_id_list.clone() {
            let _ = broadcast.send(PubsubMessage::Sync(doc_id)).await.unwrap();
        }
    }
    Ok(doc_id_list.clone())
}
