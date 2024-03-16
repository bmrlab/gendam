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
use tracing::error;

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
                    // record_type: Some(vec![SearchRecordType::Transcript]),
                    record_type: Some(vec![SearchRecordType::FrameCaption]),
                    limit: None,
                    skip: None,
                },
                ctx.get_resources_dir(),
                library.prisma_client(),
                Arc::clone(&library.qdrant_server.get_client()),
            )
            .await;

            let search_results = match res {
                Ok(res) => res,
                Err(e) => {
                    println!("error: {:?}", e);
                    return Err(rspc::Error::new(
                        rspc::ErrorCode::InternalServerError,
                        format!("failed to search: {}", e),
                    ));
                }
            };

            let file_identifiers = search_results
                .iter()
                .map(|SearchResult { file_identifier, .. }| {
                    file_identifier.clone()
                })
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
            let mut tasks_hash_map =
                std::collections::HashMap::<String, &asset_object::Data>::new();
            asset_objects.iter().for_each(|asset_object_data| {
                let hash = asset_object_data.hash.clone();
                tasks_hash_map.insert(hash, asset_object_data);
            });

            #[derive(Serialize, Type)]
            #[serde(rename_all = "camelCase")]
            pub struct SearchResultPayload {
                pub name: String,
                pub materialized_path: String,
                pub asset_object_id: i32,
                pub asset_object_hash: String,
                // #[serde(rename = "startTime")]
                pub start_time: i32,
            }

            let search_result = search_results
                .iter()
                .map(|SearchResult { file_identifier, start_timestamp, .. }| {
                    let asset_object_data = match tasks_hash_map.get(file_identifier) {
                        Some(asset_object_data) => asset_object_data.to_owned(),
                        None => {
                            error!("failed to find asset object data for file_identifier: {}", file_identifier);
                            return SearchResultPayload {
                                name: "".to_string(),
                                materialized_path: "".to_string(),
                                asset_object_id: 0,
                                asset_object_hash: "".to_string(),
                                start_time: 0,
                            };
                        }
                    };
                    let (materialized_path, name) = match asset_object_data.file_paths {
                        Some(ref file_paths) => {
                            if file_paths.len() > 0 {
                                let file_path = file_paths[0].clone();
                                (file_path.materialized_path.clone(), file_path.name.clone())
                            } else {
                                ("".to_string(), "".to_string())
                            }
                        },
                        None => ("".to_string(), "".to_string()),
                    };
                    let asset_object_hash = asset_object_data.hash.clone();
                    let asset_object_id = asset_object_data.id;
                    SearchResultPayload {
                        name,
                        materialized_path,
                        asset_object_id,
                        asset_object_hash,
                        start_time: (*start_timestamp).clone(),
                    }
                })
                .collect::<Vec<SearchResultPayload>>();
            Ok(search_result)
        }),
    )
}
