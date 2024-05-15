mod association;

use crate::routes::crr::association::{get_ids_for_dir, get_ids_for_file};
use crate::CtxWithLibrary;
use crdt::sync::FileSync;
use rspc::{Router, RouterBuilder};
use serde::Deserialize;
use specta::Type;
use tracing::{debug, info};

pub fn get_routes<TCtx>() -> RouterBuilder<TCtx>
where
    TCtx: CtxWithLibrary + Clone + Send + Sync + 'static,
{
    Router::<TCtx>::new()
        .mutation("pull", |t| {
            t(|ctx, file_path_id: String| async move {
                debug!("pull payload: {:?}", file_path_id);

                let library = ctx.library()?;

                let (file_path_id, asset_object_id, media_data_id) =
                    get_ids_for_file(library.prisma_client(), file_path_id.clone()).await?;

                debug!(
                    "file_path_id: {:?}, asset_object_id: {:?}, media_data_id: {:?}",
                    file_path_id, asset_object_id, media_data_id
                );

                let file_sync = FileSync::new(library.db_path());

                // TODO: replace db_version with real value
                let changes = file_sync
                    .pull_file_changes(0, asset_object_id, file_path_id, media_data_id)
                    .expect("Failed to pull asset object changes");

                debug!("pull changes: {changes:?}");

                Ok(changes)
            })
        })
        .mutation("pull_dir", |t| {
            #[derive(Deserialize, Type, Debug)]
            #[serde(rename_all = "camelCase")]
            struct PullDirPayload {
                dir: String,
                dir_file_path_id: String,
            }

            t(|ctx, payload: PullDirPayload| async move {
                debug!("pull_dir payload: {:?}", payload);
                let library = ctx.library()?;
                let mut ids = get_ids_for_dir(library.prisma_client(), payload.dir).await?;
                // add dir file path id
                ids.0.push(payload.dir_file_path_id);
                debug!("ids: {:?}", ids);

                let file_sync = FileSync::new(library.db_path());

                // TODO: replace db_version with real value
                let changes = file_sync.pull_dir_changes(0, ids.0, ids.1, ids.2);

                Ok(changes.map_err(|e| {
                    rspc::Error::new(
                        rspc::ErrorCode::InternalServerError,
                        format!("failed to pull dir changes: {e}"),
                    )
                })?)
            })
        })
        .mutation("apply", |t| {
            t(|ctx, changes: String| async move {
                info!("api changes: {:?}", changes.clone());
                let library = ctx.library()?;

                let mut file_sync = FileSync::new(library.db_path());
                file_sync
                    .apple_changes(changes)
                    .expect("Failed to apply changes");
                Ok(())
            })
        })
}
