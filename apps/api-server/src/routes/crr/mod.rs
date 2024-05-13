mod association;

use crate::routes::crr::association::get_ids;
use crate::CtxWithLibrary;
use crdt::sync::FileSync;
use prisma_lib::{file_path, media_data};
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
                debug!("asset_object_id: {:?}", file_path_id);

                let library = ctx.library()?;

                let file_path = library
                    .prisma_client()
                    .file_path()
                    .find_unique(file_path::UniqueWhereParam::IdEquals(file_path_id.clone()))
                    .exec()
                    .await?;

                let asset_object_id = file_path.clone().unwrap().asset_object_id;

                let media_data = library
                    .prisma_client()
                    .media_data()
                    .find_first(vec![media_data::WhereParam::AssetObjectId(
                        prisma_lib::read_filters::StringNullableFilter::Equals(
                            asset_object_id.clone(),
                        ),
                    )])
                    .exec()
                    .await?;

                let media_data_id = media_data.clone().unwrap().id;

                tracing::warn!("file_path_id: {:?}", file_path_id.clone());
                tracing::warn!("asset_object_id: {:?}", asset_object_id.clone());
                tracing::warn!("media_data_id: {:?}", media_data_id.clone());

                let file_sync = FileSync::new(library.db_path());

                let changes = file_sync
                    .pull_asset_object_changes(
                        0,
                        asset_object_id.expect("asset_object_id is None"),
                        file_path_id,
                        media_data_id,
                    )
                    .expect("Failed to pull asset object changes");

                info!("changes: {:?}", changes);

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
                let mut ids = get_ids(ctx.library()?.prisma_client(), payload.dir).await?;
                // add dir file path id
                ids.0.push(payload.dir_file_path_id);
                debug!("ids: {:?}", ids);

                Ok(())
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
