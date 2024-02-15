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
use file_handler::{
    // handle_search,
    SearchRequest,
    SearchResult,
    search_payload::{
        SearchPayload,
        SearchRecordType,
    }
};
use specta::Type;
use serde::Serialize;

pub fn get_routes() -> Router<Ctx> {
    R.router()
        .procedure(
            "all",
            R.query(move |ctx: Ctx, input: String| async move {
                let res = file_handler::handle_search(
                    SearchRequest {
                        text: input,
                        record_type: Some(SearchRecordType::Frame),
                        skip: None,
                        limit: None
                    },
                    ctx.resources_dir
                ).await.unwrap();
                // .map_err(|_| ())
                // serde_json::to_value(res).unwrap()


                #[derive(Serialize, Type)]
                pub struct SearchResultPayload {
                    #[serde(rename = "fullPath")]
                    pub full_path: String,
                }

                res.iter().map(|SearchResult { payload, score: _ }| {
                    if let SearchPayload::Frame(payload) = payload {
                        let full_path = format!("{}/frames/{}", &payload.file_identifier, &payload.frame_filename);
                        // let full_path = ctx.local_data_dir.join(full_path).display().to_string();
                        SearchResultPayload {
                            full_path: full_path,
                        }
                    } else {
                        SearchResultPayload {
                            full_path: "".to_string(),
                        }
                    }
                }).collect::<Vec<SearchResultPayload>>()
            })
        )
}
