pub mod search;
pub mod task;

use rspc::Router;
use crate::{Ctx, R};

pub fn get_routes() -> Router<Ctx> {
    R.router()
        .merge("tasks", task::get_routes())
        .merge("search", search::get_routes())
}
