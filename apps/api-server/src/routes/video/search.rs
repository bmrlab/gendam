use crate::task_queue::VideoTaskType;
use std::sync::Arc;
// use crate::{Ctx, R};
use crate::CtxWithLibrary;
use file_handler::{
    search::{SearchRequest, SearchResult},
    SearchRecordType,
};
use prisma_lib::video_task;
use qdrant_client::client::QdrantClient;
use rspc::{Router, Rspc};
use serde::Serialize;
use specta::Type;
use tracing::warn;
use vector_db::QdrantParams;

pub fn get_routes<TCtx>() -> Router<TCtx>
where
    TCtx: CtxWithLibrary + Clone + Send + Sync + 'static,
{
    Rspc::<TCtx>::new().router().procedure(
        "all",
        Rspc::<TCtx>::new().query(move |ctx: TCtx, input: String| async move {
            let library = ctx.library()?;

            warn!("start updating qdrant");
            let qdrant_channel = ctx.get_qdrant_channel();
            qdrant_channel
                .update(QdrantParams {
                    dir: library.qdrant_dir.clone(),
                    http_port: None,
                    grpc_port: None,
                })
                .await
                .expect("failed to update qdrant");
            let qdrant_url = qdrant_channel.get_url().await;

            let qdrant = Arc::new(
                QdrantClient::from_url(&qdrant_url)
                    .build()
                    .expect("failed to build qdrant client"),
            );
            warn!("finish updating qdrant");

            let res = file_handler::search::handle_search(
                SearchRequest {
                    text: input,
                    record_type: Some(vec![SearchRecordType::FrameCaption]),
                    limit: None,
                    skip: None,
                },
                ctx.get_resources_dir(),
                Arc::clone(&library.prisma_client),
                qdrant,
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

            let client_r = library.prisma_client.read().await;
            let tasks = client_r
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

            let search_result = res.iter()
                .map(
                    |SearchResult {
                         file_identifier,
                         start_timestamp,
                         ..
                     }| {
                        // TODO current version only support frame type
                        let image_path =
                            format!("{}/frames/{}.png", &file_identifier, &start_timestamp);
                        let image_path =
                            library.artifacts_dir.join(image_path).display().to_string();
                        let video_path = tasks_hash_map
                            .get(file_identifier)
                            .unwrap_or(&"".to_string())
                            .clone();
                        SearchResultPayload {
                            image_path,
                            video_path,
                            start_time: (*start_timestamp).clone(),
                        }
                    },
                )
                .collect::<Vec<SearchResultPayload>>();
            Ok(search_result)
        }),
    )
}
