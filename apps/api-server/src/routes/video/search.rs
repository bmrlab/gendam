use rspc::Router;
// use tracing::{
//     // debug,
//     info,
//     error
// };
use crate::{Ctx, R};
use prisma_lib::{
    // PrismaClient,
    new_client,
    video_task,
    VideoTaskType,
};
use file_handler::{
    // handle_search,
    search_payload::{
        SearchPayload,
        SearchRecordType,
    },
    SearchRequest,
    SearchResult
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
                ).await;
                // .unwrap();
                // .map_err(|_| ())
                // serde_json::to_value(res).unwrap()
                let res = match res {
                    Ok(res) => res,
                    Err(e) => {
                        println!("error: {:?}", e);
                        return vec![];
                    }
                };

                let file_identifiers = res.iter().filter_map(
                    |SearchResult { payload, score: _ }| {
                        if let SearchPayload::Frame(payload) = payload {
                            Some(payload.file_identifier.clone())
                        } else {
                            None
                        }
                    }
                // ).collect::<Vec<String>>();
                ).fold(Vec::new(), |mut acc, x| {
                    if !acc.contains(&x) {
                        acc.push(x);
                    }
                    acc
                });

                // println!("file_identifiers: {:?}", file_identifiers);

                let client = new_client().await.expect("failed to create prisma client");
                let tasks = client.video_task().find_many(
                    vec![
                        video_task::video_file_hash::in_vec(file_identifiers),
                        video_task::task_type::equals(VideoTaskType::Frame),
                    ]
                ).exec().await.expect("failed to list video frames");
                // println!("tasks: {:?}", tasks);
                let mut tasks_hash_map: std::collections::HashMap<String, String> = std::collections::HashMap::new();
                tasks.iter().for_each(|task| {
                    tasks_hash_map.insert(task.video_file_hash.clone(), task.video_path.clone());
                });

                #[derive(Serialize, Type)]
                pub struct SearchResultPayload {
                    #[serde(rename = "imagePath")]
                    pub image_path: String,
                    #[serde(rename = "videoPath")]
                    pub video_path: String,
                    #[serde(rename = "startTime")]
                    pub start_time: i32,
                }

                res.iter().map(|SearchResult { payload, score: _ }| {
                    if let SearchPayload::Frame(payload) = payload {
                        let image_path = format!("{}/frames/{}", &payload.file_identifier, &payload.frame_filename);
                        // let full_path = ctx.local_data_dir.join(full_path).display().to_string();
                        let video_path = tasks_hash_map.get(&payload.file_identifier).unwrap_or(&"".to_string()).clone();
                        let start_time: i32 = payload.timestamp as i32;
                        SearchResultPayload {
                            image_path,
                            video_path,
                            start_time,
                        }
                    } else {
                        SearchResultPayload {
                            image_path: "".to_string(),
                            video_path: "".to_string(),
                            start_time: 0,
                        }
                    }
                }).collect::<Vec<SearchResultPayload>>()
            })
        )
}
