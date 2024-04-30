use crate::{
    sync::{
        file::{move_file_crdt, update_file_crdt},
        folder::{move_folder_crdt, update_doc_add_new_file, update_folder_crdt},
        Folder, Item,
    },
    utils::path::get_suffix_path,
};

use super::types::FilePathRequestPayload;
use autosurgeon::{hydrate, reconcile};
use content_library::Library;
use prisma_client_rust::{raw, PrismaValue, QueryError};
use prisma_lib::file_path;

pub async fn rename_file_path(
    library: &Library,
    id: i32,
    is_dir: bool,
    materialized_path: &str,
    old_name: &str,
    new_name: &str,
) -> Result<(), rspc::Error> {
    // TODO: 所有 SQL 要放进一个 transaction 里面
    let file_path_data = library
        .prisma_client()
        .file_path()
        .find_first(vec![
            file_path::id::equals(id),
            file_path::materialized_path::equals(materialized_path.to_string()),
            file_path::is_dir::equals(is_dir),
            file_path::name::equals(old_name.to_string()),
        ])
        .exec()
        .await
        .map_err(|e| {
            rspc::Error::new(
                rspc::ErrorCode::InternalServerError,
                format!("failed to find file_path: {}", e),
            )
        })?;
    if let None = file_path_data {
        return Err(rspc::Error::new(
            rspc::ErrorCode::NotFound,
            String::from("file_path not found"),
        ));
    }

    // 如果新名字已经有冲突的了
    let same_file_data = library
        .prisma_client()
        .file_path()
        .find_unique(file_path::UniqueWhereParam::MaterializedPathNameEquals(
            materialized_path.to_string(),
            new_name.to_string(),
        ))
        .exec()
        .await
        .map_err(|e| {
            rspc::Error::new(
                rspc::ErrorCode::InternalServerError,
                format!("failed to find file_path: {}", e),
            )
        })?;

    if let Some(_) = same_file_data {
        return Err(rspc::Error::new(
            rspc::ErrorCode::InternalServerError,
            String::from("There's already a file with that name"),
        ));
    };

    // 在修改前触发
    if !is_dir {
        update_file_crdt(
            &library,
            format!("{}{}", materialized_path, old_name),
            Some(new_name.to_string()),
        )
        .await
        .expect("fail update file crdt");
    } else {
        // 再更新文件夹的文档
        let old_path = format!("{}{}", &materialized_path, &old_name);
        update_folder_crdt(&library, old_path, Some(new_name.to_string()))
            .await
            .expect("fail update file crdt");
    }

    library
        .prisma_client()
        .file_path()
        .update(
            file_path::materialized_path_name(materialized_path.to_string(), old_name.to_string()),
            vec![file_path::name::set(new_name.to_string())],
        )
        .exec()
        .await
        .map_err(|e| {
            rspc::Error::new(
                rspc::ErrorCode::InternalServerError,
                format!("failed to rename file_path item: {}", e),
            )
        })?;

    // 要区分一下是文件夹重命名还是文件重命名，如果是文件，下面的不需要
    if !is_dir {
        return Ok(());
    }

    /*
     * https://github.com/bmrlab/tauri-dam-test-playground/issues/15#issuecomment-2001923972
     */
    let old_materialized_path = format!("{}{}/", &materialized_path, &old_name);
    let new_materialized_path = format!("{}{}/", &materialized_path, &new_name);
    let old_materialized_path_like = format!("{}%", &old_materialized_path);
    library
        .prisma_client()
        ._execute_raw(raw!(
            r#"
            UPDATE FilePath
            SET materializedPath = $1 || SUBSTR(materializedPath, LENGTH($2) + 1)
            WHERE materializedPath LIKE $3
            "#,
            // 注意，这里的顺序一定要 $1,$2,$3, 序号似乎没有被遵守
            PrismaValue::String(new_materialized_path),
            PrismaValue::String(old_materialized_path.clone()),
            PrismaValue::String(old_materialized_path_like)
        ))
        // .update_many(
        //     vec![file_path::materialized_path::starts_with(old_materialized_path)],
        //     vec![file_path::materialized_path::set(new_materialized_path)],
        // )
        .exec()
        .await
        .map_err(|e| {
            rspc::Error::new(
                rspc::ErrorCode::InternalServerError,
                format!("failed to rename file_path for children: {}", e),
            )
        })?;
    Ok(())
}

pub async fn move_file_path(
    library: &Library,
    active: FilePathRequestPayload,
    target: Option<FilePathRequestPayload>,
) -> Result<(), rspc::Error> {
    // TODO: 所有 SQL 要放进一个 transaction 里面

    if let Some(target) = target.as_ref() {
        let target_full_path = format!(
            "{}{}/",
            target.materialized_path.as_str(),
            target.name.as_str()
        );
        let active_full_path = format!(
            "{}{}/",
            active.materialized_path.as_str(),
            active.name.as_str()
        );
        if target_full_path.starts_with(&active_full_path) {
            return Err(rspc::Error::new(
                rspc::ErrorCode::BadRequest,
                String::from("target is a subfolder of active"),
            ));
        }
    }

    let sql_error = |e: QueryError| {
        rspc::Error::new(
            rspc::ErrorCode::InternalServerError,
            format!("sql query failed: {}", e),
        )
    };

    let active_file_path_data = library
        .prisma_client()
        .file_path()
        .find_first(vec![
            file_path::id::equals(active.id),
            file_path::materialized_path::equals(active.materialized_path.clone()),
            file_path::is_dir::equals(active.is_dir),
            file_path::name::equals(active.name.clone()),
        ])
        .exec()
        .await
        .map_err(sql_error)?;
    let active_file_path_data = match active_file_path_data {
        Some(t) => t,
        None => {
            return Err(rspc::Error::new(
                rspc::ErrorCode::NotFound,
                String::from("active file_path not found"),
            ));
        }
    };

    if let Some(target) = target.as_ref() {
        // TODO: 首先，确保 target.is_dir == true
        let target_file_path_data = library
            .prisma_client()
            .file_path()
            .find_first(vec![
                file_path::id::equals(target.id),
                file_path::materialized_path::equals(target.materialized_path.clone()),
                file_path::is_dir::equals(true),
                file_path::name::equals(target.name.clone()),
            ])
            .exec()
            .await
            .map_err(sql_error)?;
        let _target_file_path_data = match target_file_path_data {
            Some(t) => t,
            None => {
                return Err(rspc::Error::new(
                    rspc::ErrorCode::NotFound,
                    String::from("target file_path not found"),
                ));
            }
        };
    }

    let new_materialized_path = match target.as_ref() {
        Some(target) => format!(
            "{}{}/",
            target.materialized_path.as_str(),
            target.name.as_str()
        ),
        None => "/".to_string(),
    };
    // 确保 target 下不存在相同名字的文件，不然移动失败
    let duplicated_file_path_data = library
        .prisma_client()
        .file_path()
        .find_first(vec![
            file_path::materialized_path::equals(new_materialized_path.clone()),
            file_path::name::equals(active.name.clone()),
        ])
        .exec()
        .await
        .map_err(sql_error)?;
    if let Some(data) = duplicated_file_path_data {
        return Err(rspc::Error::new(
            rspc::ErrorCode::BadRequest,
            format!("file_path already exists: {:?}", data),
        ));
    }

    // 更新文件路径前，触发同步
    if !active.is_dir {
        // 更新移动文件的crdt
        move_file_crdt(
            &library,
            active.materialized_path.clone() + &active.name,
            Some(new_materialized_path.clone() + &active.name),
        )
        .await
        .expect("fail to move file crdt");
    } else {
        // 更新移动文件夹的crdt
        move_folder_crdt(
            &library,
            active.materialized_path.clone() + &active.name,
            Some(new_materialized_path.clone() + &active.name),
        )
        .await
        .expect("fail to move folder crdt");
    }

    // rename file_path
    library
        .prisma_client()
        .file_path()
        .update(
            file_path::id::equals(active_file_path_data.id),
            vec![file_path::materialized_path::set(
                new_materialized_path.clone(),
            )],
        )
        .exec()
        .await
        .map_err(sql_error)?;

    if !active.is_dir {
        return Ok(());
    }

    /*
     * rename children items
     * /a/aa/x
     * /a/aa/x/y1
     * /a/aa/x/y2
     *
     * /a/aa/x -> /a/bb/cc/x
     * /a/aa/x/y1 -> /a/bb/cc/x/y1
     * /a/aa/x/y2 -> /a/bb/cc/x/y2
     *
     * Same as rename
     */
    let new_materialized_path = match target.as_ref() {
        Some(target) => format!(
            "{}{}/{}/",
            target.materialized_path.as_str(),
            target.name.as_str(),
            active.name.as_str()
        ),
        None => format!("/{}/", active.name.as_str()),
    };
    let old_materialized_path = format!(
        "{}{}/",
        active.materialized_path.as_str(),
        active.name.as_str()
    );
    let old_materialized_path_like = format!("{}%", &old_materialized_path);
    library
        .prisma_client()
        ._execute_raw(raw!(
            r#"
            UPDATE FilePath
            SET materializedPath = $1 || SUBSTR(materializedPath, LENGTH($2) + 1)
            WHERE materializedPath LIKE $3
            "#,
            // 注意，这里的顺序一定要 $1,$2,$3, 序号似乎没有被遵守
            PrismaValue::String(new_materialized_path),
            PrismaValue::String(old_materialized_path),
            PrismaValue::String(old_materialized_path_like)
        ))
        .exec()
        .await
        .map_err(|e| {
            rspc::Error::new(
                rspc::ErrorCode::InternalServerError,
                format!("failed to move file_path children: {}", e),
            )
        })?;

    Ok(())
}

pub async fn update_file_path_and_doc(
    library: &Library,
    path: String,
    name: String,
    doc_id: String,
) -> Result<(), rspc::Error> {
    let sync = library.sync();
    // 查询文件路径
    let file_path_data = library
        .prisma_client()
        .file_path()
        .find_unique(file_path::UniqueWhereParam::MaterializedPathNameEquals(
            path.clone(),
            name.clone(),
        ))
        .with(file_path::asset_object::fetch())
        .exec()
        .await?;

    if let Some(file_path) = file_path_data {
        let document_id = sync
            .new_document_with_id(doc_id.clone())
            .await
            .expect("fail to new document");
        // 再更新filePath
        let _ = library
            .prisma_client()
            .file_path()
            .update(
                file_path::UniqueWhereParam::IdEquals(file_path.id),
                vec![file_path::doc_id::set(Some(doc_id.clone()))],
            )
            .exec()
            .await?;

        let _ = sync
            .request_document(document_id)
            .await
            .expect("fail to request document");
    }

    Ok(())
}

pub async fn update_folder_doc(
    library: &Library,
    path: String,
    name: String,
    doc_id: String,
) -> Result<(), rspc::Error> {
    let sync = library.sync();
    // 查询文件路径
    let file_path = library
        .prisma_client()
        .file_path()
        .find_unique(file_path::UniqueWhereParam::MaterializedPathNameEquals(
            path.clone(),
            name.clone(),
        ))
        .with(file_path::asset_object::fetch())
        .exec()
        .await?
        .expect("file path not found");

    let document_id = sync
        .new_document_with_id(doc_id.clone())
        .await
        .expect("fail to new document");

    // 再更新filePath
    let _ = library
        .prisma_client()
        .file_path()
        .update(
            file_path::UniqueWhereParam::IdEquals(file_path.id),
            vec![file_path::doc_id::set(Some(doc_id.clone()))],
        )
        .exec()
        .await?;

    let mut folder = Folder {
        name: file_path.name.clone(),
        children: Vec::new(),
    };

    let root_path = file_path.materialized_path.clone() + &file_path.name.clone() + "/";
    let prefix: String = file_path.materialized_path.clone() + &file_path.name.clone();
    let sub_file_res = library
        .prisma_client()
        .file_path()
        .find_many(vec![file_path::materialized_path::starts_with(
            root_path.clone(),
        )])
        .with(file_path::asset_object::fetch())
        .exec()
        .await
        .unwrap();

    for sub_file in sub_file_res {
        let suffix = get_suffix_path(
            &format!(
                "{}{}",
                sub_file.materialized_path.clone(),
                &sub_file.name.clone()
            ),
            &prefix.clone(),
        );

        let hash = match sub_file.asset_object.unwrap() {
            Some(data) => Some(data.as_ref().clone().hash),
            None => None,
        };

        folder.children.push(Item {
            path: suffix,
            is_dir: sub_file.is_dir,
            doc_id: sub_file.doc_id.clone(),
            hash,
        });
    }

    let doc = sync
        .request_document(document_id)
        .await
        .expect("fail to request document");

    doc.with_doc_mut(|doc| {
        let mut tx = doc.transaction();
        reconcile(&mut tx, &folder).unwrap();
        tx.commit();
    });

    let value = doc.with_doc(|doc| {
        let value: Folder = hydrate(doc).unwrap();
        value
    });

    tracing::debug!("同步结束后文件文档为: {:#?}", value);

    Ok(())
}

pub async fn update_doc_new_file(
    library: &Library,
    path: String,
    name: String,
) -> Result<Vec<String>, rspc::Error> {
    match update_doc_add_new_file(&library, path, name).await {
        Ok(doc_id_list) => Ok(doc_id_list),
        Err(_) => Err(rspc::Error::new(
            rspc::ErrorCode::InternalServerError,
            format!("fail update_doc_new_file"),
        )),
    }
}
