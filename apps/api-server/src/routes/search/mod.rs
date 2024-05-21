use crate::CtxWithLibrary;
use rspc::{Router, RouterBuilder};
mod search;
use search::{SearchRequestPayload, search_all};

pub fn get_routes<TCtx>() -> RouterBuilder<TCtx>
where
    TCtx: CtxWithLibrary + Clone + Send + Sync + 'static,
{
    Router::<TCtx>::new()
        .query("all", |t| {
            t(|ctx: TCtx, input: SearchRequestPayload| async move {
                let library = ctx.library()?;
                let qdrant_info = ctx.qdrant_info()?;
                let ai_handler = ctx.ai_handler()?;
                search_all(
                    &library,
                    &qdrant_info,
                    &ai_handler,
                    input
                ).await
            })
        })
}
