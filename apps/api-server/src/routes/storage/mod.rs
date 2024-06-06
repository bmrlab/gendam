use futures::{future::try_join_all, TryFutureExt};
use rspc::{Router, RouterBuilder};
use s3_handler::upload_to_s3;

use crate::CtxWithLibrary;

mod s3_handler;

pub fn get_routes<TCtx>() -> RouterBuilder<TCtx>
where
    TCtx: CtxWithLibrary + Clone + Send + Sync + 'static,
{
    Router::<TCtx>::new().mutation("upload_to_s3", |t| {
        t({
            |_, input: Vec<String>| async move {
                try_join_all(
                    input
                        .into_iter()
                        .map(|input| {
                            upload_to_s3(input.clone()).map_err(move |e| {
                                rspc::Error::new(
                                    rspc::ErrorCode::InternalServerError,
                                    format!(
                                        "failed to upload file with hash {} error: {}",
                                        input, e
                                    ),
                                )
                            })
                        })
                        .collect::<Vec<_>>(),
                )
            }
        })
    })
}
