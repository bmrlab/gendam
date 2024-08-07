use std::marker::PhantomData;

use crate::{error::sql_error, routes::assets::types::FilePathWithAssetObjectData};
use content_base::{
    query::{
        payload::{SearchResult, SearchResultData},
        QueryPayload,
    },
    ContentBase,
};
use content_library::Library;
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SearchRequestPayload {
    pub text: String,
}

#[derive(Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct VideoSearchResultMetadata {
    pub start_time: i32,
    pub end_time: i32,
}

#[derive(Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AudioSearchResultMetadata {
    pub start_time: i32,
    pub end_time: i32,
}

#[derive(Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ImageSearchResultMetadata {
    data: i32,
}

#[derive(Serialize, Type)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum SearchResultMetadata {
    Video(VideoSearchResultMetadata),
    Audio(AudioSearchResultMetadata),
    Image(ImageSearchResultMetadata),
}

impl From<&content_base::query::payload::SearchMetadata> for SearchResultMetadata {
    fn from(metadata: &content_base::query::payload::SearchMetadata) -> Self {
        match metadata {
            content_base::query::payload::SearchMetadata::Video(item) => {
                SearchResultMetadata::Video(VideoSearchResultMetadata {
                    start_time: item.start_timestamp as i32,
                    end_time: item.end_timestamp as i32,
                })
            }
            content_base::query::payload::SearchMetadata::Audio(item) => {
                SearchResultMetadata::Audio(AudioSearchResultMetadata {
                    start_time: item.start_timestamp as i32,
                    end_time: item.end_timestamp as i32,
                })
            }
            content_base::query::payload::SearchMetadata::Image(_) => {
                SearchResultMetadata::Image(ImageSearchResultMetadata { data: 0 })
            }
        }
    }
}

#[derive(Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SearchResultPayload {
    pub file_path: FilePathWithAssetObjectData,
    pub metadata: SearchResultMetadata,
    pub score: f32,
}

pub async fn search_all(
    library: &Library,
    content_base: &ContentBase,
    input: SearchRequestPayload,
) -> Result<Vec<SearchResultPayload>, rspc::Error> {
    let text = input.text.clone();

    let payload = QueryPayload::new(&text);
    let res = content_base.query(payload).await;
    tracing::debug!("search result: {:?}", res);

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

    let result = retrieve_assets_for_search(library, &search_results, |item, file_path| {
        SearchResultPayload {
            file_path: file_path.clone().into(),
            metadata: SearchResultMetadata::from(&item.metadata),
            score: item.score,
        }
    })
    .await?;
    Ok(result)
}

pub async fn retrieve_assets_for_search<TOriginal, TTarget, TFnConvert>(
    library: &Library,
    search_results: &[TOriginal],
    convert: TFnConvert,
) -> Result<Vec<TTarget>, rspc::Error>
where
    TOriginal: SearchResult,
    TFnConvert: Fn(&TOriginal, &prisma_lib::file_path::Data) -> TTarget,
{
    let file_identifiers = search_results
        .iter()
        .map(|v| v.file_identifier().to_string())
        .fold(Vec::new(), |mut acc, x| {
            if !acc.contains(&x) {
                acc.push(x);
            }
            acc
        });

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
        .exec()
        .await
        .map_err(sql_error)?;

    let mut tasks_hash_map =
        std::collections::HashMap::<String, &prisma_lib::asset_object::Data>::new();
    asset_objects.iter().for_each(|asset_object_data| {
        let hash = asset_object_data.hash.clone();
        tasks_hash_map.insert(hash, asset_object_data);
    });

    let results_with_asset = search_results
        .iter()
        .filter_map(|search_result| {
            let mut asset_object_data = match tasks_hash_map.get(search_result.file_identifier()) {
                Some(asset_object_data) => (**asset_object_data).clone(),
                None => {
                    tracing::error!(
                        "failed to find asset object data for file_identifier: {}",
                        search_result.file_identifier()
                    );
                    return None;
                }
            };
            let file_paths = asset_object_data.file_paths.take();
            let file_path = match file_paths {
                Some(file_paths) => {
                    if file_paths.len() > 0 {
                        let mut file_path_data = file_paths[0].clone();
                        file_path_data.asset_object = Some(Some(Box::new(asset_object_data)));
                        file_path_data
                    } else {
                        return None;
                    }
                }
                None => {
                    return None;
                }
            };

            // let result = SearchResultPayload {
            //     file_path: file_path.into(),
            //     metadata: search_result.metadata().into(),
            //     score: search_result.score(),
            // };
            let result = convert(search_result, &file_path);
            Some(result)
        })
        .collect::<Vec<_>>();

    Ok(results_with_asset)
}
