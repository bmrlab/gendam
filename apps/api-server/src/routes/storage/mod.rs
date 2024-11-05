use std::collections::HashSet;

use futures::{future::try_join_all, TryFutureExt};
use prisma_lib::{asset_object, data_location, file_path, read_filters::StringFilter};
use rspc::{Router, RouterBuilder};
use s3_handler::upload_to_s3;
use serde::Deserialize;
use specta::Type;

use crate::{library::get_library_settings, CtxWithLibrary};

pub(crate) mod location;
mod s3_handler;

pub fn get_routes<TCtx>() -> RouterBuilder<TCtx>
where
    TCtx: CtxWithLibrary + Clone + Send + Sync + 'static,
{
    Router::<TCtx>::new().mutation("upload_to_s3", |t| {
        #[derive(Deserialize, Type, Debug)]
        #[serde(rename_all = "camelCase")]
        struct UploadPayload {
            materialized_paths: Vec<String>,
            hashes: Vec<String>,
        }
        t({
            |ctx, input: UploadPayload| async move {
                let library = ctx.library()?;
                let library_settings = get_library_settings(&library.dir);
                let where_param = input
                    .materialized_paths
                    .into_iter()
                    .map(|f| {
                        file_path::WhereParam::And(vec![
                            file_path::materialized_path::starts_with(f.to_string()),
                            file_path::is_dir::equals(false),
                        ])
                    })
                    .collect::<Vec<file_path::WhereParam>>();

                let data = library
                    .prisma_client()
                    .file_path()
                    .find_many(vec![file_path::WhereParam::Or(where_param)])
                    .with(file_path::asset_object::fetch())
                    .exec()
                    .await?;

                let hashes_under_dir = data
                    .iter()
                    .filter_map(|d| {
                        if d.asset_object().is_err() {
                            return None;
                        }
                        d.asset_object().unwrap().map(|a| a.hash.to_string())
                    })
                    .collect::<Vec<String>>();

                let mut hashes = input.hashes.clone();
                hashes.extend(hashes_under_dir);
                // dedup
                let set: HashSet<String> = hashes.drain(..).collect();
                hashes.extend(set.into_iter());

                // upload to s3
                let s3_config = library_settings.s3_config;
                if s3_config.is_none() {
                    return Err(rspc::Error::new(
                        rspc::ErrorCode::PreconditionFailed,
                        "s3 config is not set".to_string(),
                    ));
                }
                try_join_all(
                    hashes
                        .clone()
                        .into_iter()
                        .map(|hash| {
                            upload_to_s3(hash.clone(), s3_config.clone().unwrap()).map_err(
                                move |e| {
                                    rspc::Error::new(
                                        rspc::ErrorCode::InternalServerError,
                                        format!(
                                            "failed to upload file with hash {} error: {}",
                                            hash, e
                                        ),
                                    )
                                },
                            )
                        })
                        .collect::<Vec<_>>(),
                )
                .await?;

                // update data location
                let client = library.prisma_client();

                // get assetObject id and hash conbination
                let mut upsert_unique_group = client
                    .asset_object()
                    .find_many(vec![asset_object::WhereParam::Hash(StringFilter::InVec(
                        input.hashes,
                    ))])
                    .exec()
                    .await?
                    .into_iter()
                    .map(|a| (a.id, a.hash))
                    .collect::<Vec<(i32, String)>>();

                data.into_iter().for_each(|d| {
                    if let Ok(Some(a)) = d.asset_object() {
                        upsert_unique_group.push((a.id, a.hash.clone()));
                    }
                });

                // crate or update data location
                let batch_statement = upsert_unique_group.into_iter().map(|u| {
                    client.data_location().upsert(
                        data_location::UniqueWhereParam::AssetObjectIdMediumEquals(
                            u.0,
                            "s3".to_string(),
                        ),
                        (
                            "s3".to_string(),
                            asset_object::UniqueWhereParam::HashEquals(u.1),
                            vec![],
                        ),
                        vec![],
                    )
                });

                library.prisma_client()._batch(batch_statement).await?;
                // rspc::Error::new(
                //     rspc::ErrorCode::InternalServerError,
                //     format!("failed to update data location error: {}", e),
                // )

                // check delete local file or not on settings
                let delete_local_or_not = library_settings.always_delete_local_file_after_upload;

                if delete_local_or_not {
                    // delete local file
                    hashes.into_iter().for_each(|h| {
                        let file = library.dir.join("files").join(&h[0..3]).join(h.clone());
                        let artifacts_dir = library.dir.join("artifacts").join(&h[0..3]).join(h);

                        match std::fs::remove_dir_all(artifacts_dir) {
                            Ok(_) => {}
                            Err(e) => tracing::error!("failed to delete local dir: {}", e),
                        }
                        match std::fs::remove_file(file) {
                            Ok(_) => {}
                            Err(e) => tracing::error!("failed to delete local file: {}", e),
                        }
                    });
                }
                Ok(())
            }
        })
    })
}
