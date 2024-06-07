use std::collections::HashSet;

use futures::{future::try_join_all, TryFutureExt};
use prisma_lib::file_path::{self};
use rspc::{Router, RouterBuilder};
use s3_handler::upload_to_s3;
use serde::Deserialize;
use specta::Type;

use crate::CtxWithLibrary;

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

                let mut hashes = input.hashes;
                hashes.extend(hashes_under_dir);
                // dedup
                let set: HashSet<String> = hashes.drain(..).collect();
                hashes.extend(set.into_iter());

                try_join_all(
                    hashes
                        .into_iter()
                        .map(|hash| {
                            upload_to_s3(hash.clone()).map_err(move |e| {
                                rspc::Error::new(
                                    rspc::ErrorCode::InternalServerError,
                                    format!(
                                        "failed to upload file with hash {} error: {}",
                                        hash, e
                                    ),
                                )
                            })
                        })
                        .collect::<Vec<_>>(),
                )
                .await?;
                Ok(())
            }
        })
    })
}
