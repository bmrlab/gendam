use crate::routes::assets::types::FilePathWithAssetObjectData;
use content_base::{
    query::{
        payload::{ContentIndexMetadata, RetrievalResultData, SearchResultData},
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
#[serde(rename_all = "camelCase")]
pub struct RawTextSearchResultMetadata {
    start_index: u32,
    end_index: u32,
}

#[derive(Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct WebPageSearchResultMetadata {
    start_index: u32,
    end_index: u32,
}

#[derive(Serialize, Type)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum SearchResultMetadata {
    Video(VideoSearchResultMetadata),
    Audio(AudioSearchResultMetadata),
    Image(ImageSearchResultMetadata),
    RawText(RawTextSearchResultMetadata),
    WebPage(WebPageSearchResultMetadata),
}

impl From<&ContentIndexMetadata> for SearchResultMetadata {
    fn from(metadata: &ContentIndexMetadata) -> Self {
        match metadata {
            ContentIndexMetadata::Video(item) => {
                SearchResultMetadata::Video(VideoSearchResultMetadata {
                    start_time: item.start_timestamp as i32,
                    end_time: item.end_timestamp as i32,
                })
            }
            ContentIndexMetadata::Audio(item) => {
                SearchResultMetadata::Audio(AudioSearchResultMetadata {
                    start_time: item.start_timestamp as i32,
                    end_time: item.end_timestamp as i32,
                })
            }
            ContentIndexMetadata::Image(_) => {
                SearchResultMetadata::Image(ImageSearchResultMetadata { data: 0 })
            }
            ContentIndexMetadata::RawText(item) => {
                SearchResultMetadata::RawText(RawTextSearchResultMetadata {
                    start_index: item.start_index as u32,
                    end_index: item.end_index as u32,
                })
            }
            ContentIndexMetadata::WebPage(item) => {
                SearchResultMetadata::WebPage(WebPageSearchResultMetadata {
                    start_index: item.start_index as u32,
                    end_index: item.end_index as u32,
                })
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
    pub highlight: Option<String>,
}

pub async fn search_all(
    library: &Library,
    content_base: &ContentBase,
    input: SearchRequestPayload,
) -> Result<Vec<SearchResultPayload>, rspc::Error> {
    let text = input.text.clone();

    let payload = QueryPayload::new(&text);
    let res = content_base.query(payload, None).await;
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
            highlight: item.highlight.clone(),
        }
    })
    .await?;
    Ok(result)
}

/// 以下是 search 和 rag 共用的辅助函数，实现一个 trait 用于统一处理两种类型的搜索结果
#[allow(dead_code)]
pub(super) trait ContentQueryResultTrait: std::fmt::Debug {
    fn file_identifier(&self) -> &str;
    fn metadata(&self) -> &ContentIndexMetadata;
    fn score(&self) -> f32;
}

impl ContentQueryResultTrait for SearchResultData {
    fn file_identifier(&self) -> &str {
        &self.file_identifier
    }
    fn metadata(&self) -> &ContentIndexMetadata {
        &self.metadata
    }
    fn score(&self) -> f32 {
        self.score
    }
}

impl ContentQueryResultTrait for RetrievalResultData {
    fn file_identifier(&self) -> &str {
        &self.file_identifier
    }
    fn metadata(&self) -> &ContentIndexMetadata {
        &self.metadata
    }
    fn score(&self) -> f32 {
        self.score
    }
}

pub async fn retrieve_assets_for_search<TOriginal, TTarget, TFnConvert>(
    library: &Library,
    search_results: &[TOriginal],
    convert: TFnConvert,
) -> Result<Vec<TTarget>, rspc::Error>
where
    TOriginal: ContentQueryResultTrait,
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
        .await?;

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
