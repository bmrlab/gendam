// use crate::task_queue::VideoTaskType;
use std::sync::Arc;
// use crate::{Ctx, R};
use crate::CtxWithLibrary;
use file_handler::{
    search::{SearchRequest, SearchResult},
    SearchRecordType,
};
use prisma_lib::asset_object;
use rspc::{Router, Rspc};
use serde::Serialize;
use specta::Type;

pub fn get_routes<TCtx>() -> Router<TCtx>
where
    TCtx: CtxWithLibrary + Clone + Send + Sync + 'static,
{
    Rspc::<TCtx>::new().router().procedure(
        "all",
        Rspc::<TCtx>::new().query(move |ctx: TCtx, input: String| async move {
            let library = ctx.library()?;

            let res = file_handler::search::handle_search(
                SearchRequest {
                    text: input,
                    record_type: Some(vec![SearchRecordType::FrameCaption]),
                    limit: None,
                    skip: None,
                },
                ctx.get_resources_dir(),
                library.prisma_client(),
                Arc::clone(&library.qdrant_server.get_client()),
            )
            .await;
            // .unwrap();
            // .map_err(|_| ())
            // serde_json::to_value(res).unwrap()
            let res = match res {
                Ok(res) => res,
                Err(e) => {
                    println!("error: {:?}", e);
                    return Err(rspc::Error::new(
                        rspc::ErrorCode::InternalServerError,
                        format!("failed to search: {}", e),
                    ));
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
            let asset_objects = library.prisma_client()
                .asset_object()
                .find_many(vec![
                    asset_object::hash::in_vec(file_identifiers)
                ])
                .with(asset_object::file_paths::fetch(vec![]))
                .exec()
                .await
                .expect("failed to list asset objects");

            // println!("tasks: {:?}", tasks);
            let mut tasks_hash_map: std::collections::HashMap<String, String> =
                std::collections::HashMap::new();
            asset_objects.iter().for_each(|asset_object_data| {
                let local_video_file_full_path = format!(
                    "{}/{}",
                    library.files_dir.to_str().unwrap(),
                    asset_object_data.id
                );
                let hash = asset_object_data.hash.clone().unwrap_or(String::from(""));
                tasks_hash_map.insert(hash, local_video_file_full_path);
            });

            #[derive(Serialize, Type)]
            pub struct SearchResultPayload {
                #[serde(rename = "videoPath")]
                pub video_path: String,
                #[serde(rename = "startTime")]
                pub start_time: i32,
            }

            let search_result = res
                .iter()
                .map(|SearchResult {
                    file_identifier,
                    start_timestamp,
                    ..
                }| {
                    let video_path = tasks_hash_map
                        .get(file_identifier)
                        .unwrap_or(&"".to_string())
                        .clone();
                    SearchResultPayload {
                        video_path,
                        start_time: (*start_timestamp).clone(),
                    }
                })
                .collect::<Vec<SearchResultPayload>>();
            Ok(search_result)
        }),
    )
}
