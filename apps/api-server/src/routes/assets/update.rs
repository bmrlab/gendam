use super::types::FilePathRequestPayload;
use content_library::Library;
use prisma_client_rust::{raw, PrismaValue, QueryError};
use prisma_lib::file_path;

pub async fn rename_file_path(
    library: &Library,
    id: String,
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
            file_path::id::equals(id.clone()),
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

    library
        .prisma_client()
        .file_path()
        .update(
            file_path::id::equals(id.clone()),
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
            PrismaValue::String(old_materialized_path),
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
                file_path::id::equals(target.id.clone()),
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
