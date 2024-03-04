pub mod search;
pub mod task;

use rspc::{Rspc, Router};
// use crate::{Ctx, R};
use crate::CtxWithLibrary;

pub fn get_routes<TCtx>() -> Router<TCtx>
where TCtx: CtxWithLibrary + Clone + Send + Sync + 'static
{
    Rspc::<TCtx>::new().router()
        .merge("tasks", task::get_routes::<TCtx>())
        .merge("search", search::get_routes::<TCtx>())
}
