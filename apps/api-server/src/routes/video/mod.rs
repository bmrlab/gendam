pub mod search;
pub mod task;

use crate::CtxWithLibrary;
use rspc::{Router, RouterBuilder};

pub fn get_routes<TCtx>() -> RouterBuilder<TCtx>
where
    TCtx: CtxWithLibrary + Clone + Send + Sync + 'static,
{
    Router::new()
        .merge("tasks.", task::get_routes::<TCtx>())
        .merge("search.", search::get_routes::<TCtx>())
}
