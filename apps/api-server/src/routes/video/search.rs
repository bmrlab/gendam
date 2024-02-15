use rspc::Router;
// use tracing::{
//     // debug,
//     info,
//     error
// };
use crate::{Ctx, R};
// use prisma_lib::{
//     PrismaClient,
//     new_client,
// };
// use file_handler::video::VideoHandler;
// use specta::Type;
// use serde::Serialize;

pub fn get_routes() -> Router<Ctx> {
    R.router()
        .procedure(
            "all",
            R.query(move |ctx: Ctx, input: String| async move {
                let res = file_handler::handle_search(
                    file_handler::SearchRequest {
                        text: input,
                        record_type: Some(file_handler::search_payload::SearchRecordType::Frame),
                        skip: None,
                        limit: None
                    },
                    ctx.resources_dir
                ).await.unwrap();
                // .map_err(|_| ())
                serde_json::to_value(res).unwrap()
            })
        )
}
