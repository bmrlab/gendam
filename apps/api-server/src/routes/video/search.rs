use super::task::VideoTaskType;
use crate::{Ctx, R};
use file_handler::search::{SearchRecordType, SearchRequest, SearchResult};
use prisma_lib::{new_client_with_url, video_task};
use rspc::Router;
use serde::Serialize;
use specta::Type;

pub fn get_routes() -> Router<Ctx> {
    R.router().procedure(
        "all",
        R.query(move |ctx: Ctx, input: String| async move {
            let res = file_handler::search::handle_search(
                SearchRequest {
                    text: input,
                    record_type: Some(vec![SearchRecordType::Frame]),
                    limit: None,
                },
                ctx.resources_dir,
                ctx.library.clone(),
            )
            .await;
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

            let file_identifiers = res
                .iter()
                .map(
                    |SearchResult {
                         file_identifier, ..
                     }| file_identifier.clone(),
                )
                .fold(Vec::new(), |mut acc, x| {
                    if !acc.contains(&x) {
                        acc.push(x);
                    }
                    acc
                });

            // println!("file_identifiers: {:?}", file_identifiers);

            let client = new_client_with_url(ctx.library.db_url.as_str())
                .await
                .expect("failed to create prisma client");
            client._db_push().await.expect("failed to push db"); // apply migrations
            let tasks = client
                .video_task()
                .find_many(vec![
                    video_task::video_file_hash::in_vec(file_identifiers),
                    video_task::task_type::equals(VideoTaskType::Frame.to_string()),
                ])
                .exec()
                .await
                .expect("failed to list video frames");
            // println!("tasks: {:?}", tasks);
            let mut tasks_hash_map: std::collections::HashMap<String, String> =
                std::collections::HashMap::new();
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

            res.iter().map(
                |SearchResult { file_identifier, start_timestamp, ..}| {
                    // TODO current version only support frame type
                    let image_path =
                        format!("{}/frames/{}.png", &file_identifier, &start_timestamp);
                    let image_path = ctx.library.dir.join(image_path).display().to_string();
                    let video_path = tasks_hash_map
                        .get(file_identifier)
                        .unwrap_or(&"".to_string())
                        .clone();
                    SearchResultPayload {
                        image_path,
                        video_path,
                        start_time: (*start_timestamp).clone(),
                    }
                }
            )
            .collect::<Vec<SearchResultPayload>>()
        }),
    )
}
