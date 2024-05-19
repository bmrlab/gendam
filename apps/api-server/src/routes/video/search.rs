use crate::CtxWithLibrary;
use file_handler::{
    search::{SearchRequest, SearchResult},
    SearchRecordType,
};
use rspc::{Router, RouterBuilder};
use serde::{Deserialize, Serialize};
use specta::Type;

fn sql_error(e: prisma_client_rust::QueryError) -> rspc::Error {
    rspc::Error::new(
        rspc::ErrorCode::InternalServerError,
        format!("sql query failed: {}", e),
    )
}

pub fn get_routes<TCtx>() -> RouterBuilder<TCtx>
where
    TCtx: CtxWithLibrary + Clone + Send + Sync + 'static,
{
    Router::<TCtx>::new().query("all", |t| {
        #[derive(Deserialize, Type)]
        #[serde(rename_all = "camelCase")]
        pub struct SearchRequestPayload {
            pub text: String,
            pub record_type: String,
        }
        #[derive(Serialize, Type)]
        #[serde(rename_all = "camelCase")]
        pub struct SearchResultMetadata {
            // #[serde(rename = "startTime")]
            pub start_time: i32,
            pub end_time: i32,
            pub score: f32,
        }
        #[derive(Serialize, Type)]
        #[serde(rename_all = "camelCase")]
        pub struct SearchResultPayload {
            file_path: prisma_lib::file_path::Data,
            metadata: SearchResultMetadata,
        }
        t(move |ctx: TCtx, input: SearchRequestPayload| async move {
            let library = ctx.library()?;
            let qdrant_info = ctx.qdrant_info()?;

            let text = input.text.clone();
            let record_types = match input.record_type {
                s if s == "Transcript" => vec![SearchRecordType::Transcript],
                s if s == "FrameCaption" => {
                    vec![SearchRecordType::Frame, SearchRecordType::FrameCaption]
                }
                s if s == "Frame" => vec![SearchRecordType::Frame, SearchRecordType::FrameCaption],
                _ => {
                    return Err(rspc::Error::new(
                        rspc::ErrorCode::BadRequest,
                        "invalid record_type".to_string(),
                    ))
                }
            };
            let res = file_handler::search::handle_search(
                SearchRequest {
                    text,
                    record_type: Some(record_types),
                },
                library.qdrant_client(),
                &qdrant_info.vision_collection.name,
                &qdrant_info.language_collection.name,
                ctx.ai_handler()?.multi_modal_embedding.as_ref(),
                ctx.ai_handler()?.text_embedding.as_ref(),
            )
            .await;

            // debug!("search result: {:?}", res);

            let search_results = match res {
                Ok(res) => res,
                Err(e) => {
                    tracing::error!("failed to search: {}", e);
                    return Err(rspc::Error::new(
                        rspc::ErrorCode::InternalServerError,
                        format!("failed to search: {}", e),
                    ));
                }
            };

            let file_identifiers = search_results
                .iter()
                .map(
                    |SearchResult {
                         file_identifier, ..
                     }| { file_identifier.clone() },
                )
                .fold(Vec::new(), |mut acc, x| {
                    if !acc.contains(&x) {
                        acc.push(x);
                    }
                    acc
                });

            // println!("file_identifiers: {:?}", file_identifiers);
            let asset_objects = library
                .prisma_client()
                .asset_object()
                .find_many(vec![prisma_lib::asset_object::hash::in_vec(
                    file_identifiers,
                )])
                .with(
                    prisma_lib::asset_object::file_paths::fetch(vec![])
                        .order_by(prisma_lib::file_path::created_at::order(
                            prisma_client_rust::Direction::Desc,
                        ))
                        .take(1),
                )
                .with(prisma_lib::asset_object::media_data::fetch())
                .exec()
                .await
                .map_err(sql_error)?;

            // println!("tasks: {:?}", tasks);
            let mut tasks_hash_map =
                std::collections::HashMap::<String, &prisma_lib::asset_object::Data>::new();
            asset_objects.iter().for_each(|asset_object_data| {
                let hash = asset_object_data.hash.clone();
                tasks_hash_map.insert(hash, asset_object_data);
            });

            let search_result = search_results
                .iter()
                .filter_map(|search_result| {
                    let SearchResult {
                        file_identifier,
                        start_timestamp,
                        end_timestamp,
                        score,
                        ..
                    } = search_result;
                    let metadata = SearchResultMetadata {
                        start_time: (*start_timestamp).clone(),
                        end_time: (*end_timestamp).clone(),
                        score: *score,
                    };
                    let mut asset_object_data = match tasks_hash_map.get(file_identifier) {
                        Some(asset_object_data) => (**asset_object_data).clone(),
                        None => {
                            tracing::error!(
                                "failed to find asset object data for file_identifier: {}",
                                file_identifier
                            );
                            return None;
                        }
                    };
                    let file_paths = asset_object_data.file_paths.take();
                    let file_path = match file_paths {
                        Some(file_paths) => {
                            if file_paths.len() > 0 {
                                let mut file_path_data = file_paths[0].clone();
                                file_path_data.asset_object =
                                    Some(Some(Box::new(asset_object_data)));
                                file_path_data
                            } else {
                                return None;
                            }
                        }
                        None => {
                            return None;
                        }
                    };
                    let result = SearchResultPayload {
                        file_path,
                        metadata,
                    };
                    Some(result)
                })
                .collect::<Vec<_>>();
            Ok(search_result)
        })
    })
}
