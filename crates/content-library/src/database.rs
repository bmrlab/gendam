use prisma_lib::{asset_object, file_path, media_data, new_client_with_url, PrismaClient};
use std::path::PathBuf;

async fn copy_data_from_legacy_db(
    client: &PrismaClient,
    legacy_client: &PrismaClient,
) -> Result<(), ()> {
    // copy AssetObject, MediaData, FilePath from legacy db to new db
    client
        ._transaction()
        .run(|client| async move {
            // asset_object
            let legacy_asset_objects = legacy_client
                .asset_object()
                .find_many(vec![])
                .exec()
                .await?;
            for data in legacy_asset_objects {
                client
                    .asset_object()
                    .create(vec![
                        asset_object::hash::set(data.hash),
                        asset_object::size::set(data.size),
                        asset_object::mime_type::set(data.mime_type),
                    ])
                    .exec()
                    .await?;
            }
            // media_data
            let legacy_media_data = legacy_client.media_data().find_many(vec![]).exec().await?;
            for data in legacy_media_data {
                client
                    .media_data()
                    .create(vec![
                        media_data::asset_object_id::set(data.asset_object_id),
                        media_data::width::set(data.width),
                        media_data::height::set(data.height),
                        media_data::duration::set(data.duration),
                        media_data::bit_rate::set(data.bit_rate),
                        media_data::has_audio::set(data.has_audio),
                    ])
                    .exec()
                    .await?;
            }
            // file_path
            let legacy_file_paths = legacy_client.file_path().find_many(vec![]).exec().await?;
            for data in legacy_file_paths {
                client
                    .file_path()
                    .create(vec![
                        file_path::is_dir::set(data.is_dir),
                        file_path::materialized_path::set(data.materialized_path),
                        file_path::name::set(data.name),
                        file_path::asset_object_id::set(data.asset_object_id),
                    ])
                    .exec()
                    .await?;
            }
            // file_handler_task
            let legacy_file_handler_tasks = legacy_client
                .file_handler_task()
                .find_many(vec![])
                .exec()
                .await?;
            for data in legacy_file_handler_tasks {
                client
                    .file_handler_task()
                    .create(
                        data.asset_object_id,
                        data.task_type,
                        vec![], // 不设置 exitCode, startsAt, endsAt, 所有 task 需要重新执行
                    )
                    .exec()
                    .await?;
            }
            Ok::<(), prisma_client_rust::QueryError>(())
        })
        .await
        .map_err(|e| {
            tracing::error!("failed to copy legacy data: {}", e);
            ()
        })?;
    Ok(())
}

pub async fn migrate_library(db_dir: &PathBuf) -> Result<PrismaClient, ()> {
    let db_url = format!(
        // "file:{}?socket_timeout=1&connection_limit=10",
        "file:{}?socket_timeout=15&connection_limit=1",
        db_dir.join("gendam-library.db").to_str().unwrap()
    );
    let client = new_client_with_url(db_url.as_str()).await.map_err(|_e| {
        tracing::error!("failed to create prisma client");
    })?;

    client
        ._migrate_deploy()
        // ._db_push()
        // .accept_data_loss() // --accept-data-loss in CLI
        // .force_reset()      // --force-reset in CLI
        .await // apply migrations
        .map_err(|e| {
            tracing::error!("failed to deploy db migrations: {}", e);
        })?;

    let legacy_db_path = db_dir.join("library.db");
    if legacy_db_path.exists() {
        tracing::info!("db file not found, copy data from legacy db");
        let legacy_db_url = format!(
            "file:{}?socket_timeout=15&connection_limit=1",
            legacy_db_path.to_str().unwrap()
        );
        let legacy_client = new_client_with_url(legacy_db_url.as_str())
            .await
            .map_err(|_e| {
                tracing::error!("failed to create prisma client");
            })?;
        // copy AssetObject, MediaData, FilePath from legacy db to new db
        copy_data_from_legacy_db(&client, &legacy_client).await?;
        // rename legacy db file to library.db.archived
        if let Err(e) = std::fs::rename(&legacy_db_path, db_dir.join("library.db.archived")) {
            // remove db file
            tracing::error!("failed to rename legacy db file: {}", e);
            if let Err(e) = std::fs::remove_file(&legacy_db_path) {
                tracing::error!("failed to remove legacy db file: {}", e);
            }
        }
    }

    Ok(client)
}
