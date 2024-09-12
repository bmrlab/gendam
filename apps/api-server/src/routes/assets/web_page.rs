use crate::validators;
use blake3::Hasher;
use content_base::{metadata::web_page::WebPageMetadata, ContentMetadata};
use content_handler::web_page::fetch_url;
use content_library::Library;
use global_variable::get_current_fs_storage;
use prisma_client_rust::QueryError;
use prisma_lib::{asset_object, file_path};
use storage::Storage;

pub async fn process_web_page(
    library: &Library,
    materialized_path: &str,
    url: &str,
) -> Result<(file_path::Data, asset_object::Data, bool), rspc::Error> {
    tracing::debug!("process_web_page: {}", url);

    let (title, html, screenshot) = fetch_url(url).await.map_err(|e| {
        tracing::error!("Failed to fetch url: {}", e);
        rspc::Error::new(
            rspc::ErrorCode::InternalServerError,
            format!("Failed to fetch url: {}", e),
        )
    })?;

    tracing::debug!("get data from url: {:?}", title);

    let mut hasher = Hasher::new();
    hasher.update(html.as_bytes());
    let html_hash = hasher.finalize().to_hex()[..16].to_string();

    let storage = get_current_fs_storage!().map_err(|e| {
        rspc::Error::new(
            rspc::ErrorCode::InternalServerError,
            format!("failed to get current storage: {}", e),
        )
    })?;

    let html_title = title.unwrap_or("".to_string());

    let html_data: Vec<u8> = html.as_bytes().into();

    storage
        .write(library.relative_file_path(&html_hash), html_data.into())
        .await
        .map_err(|e| {
            rspc::Error::new(
                rspc::ErrorCode::InternalServerError,
                format!("failed to write file: {}", e),
            )
        })?;

    storage
        .write(
            library
                .relative_artifacts_path(&html_hash)
                .join("thumbnail.png"),
            screenshot.into(),
        )
        .await
        .map_err(|e| {
            rspc::Error::new(
                rspc::ErrorCode::InternalServerError,
                format!("failed to write file: {}", e),
            )
        })?;

    let (asset_object_data, file_path_data, asset_object_existed) = library
        .prisma_client()
        ._transaction()
        .run(|client| async move {
            let mut asset_object_existed = false;

            let asset_object_data = match client
                .asset_object()
                .find_unique(asset_object::hash::equals(html_hash.clone()))
                .exec()
                .await?
            {
                Some(asset_object_data) => {
                    asset_object_existed = true;
                    asset_object_data
                }
                None => {
                    client
                        .asset_object()
                        .create(
                            html_hash.clone(),
                            0,
                            vec![
                                asset_object::media_data::set(Some(
                                    serde_json::to_string(&ContentMetadata::WebPage(
                                        WebPageMetadata {
                                            source_url: url.to_string(),
                                        },
                                    ))
                                    .expect("WebPageMetadata can be converted to string"),
                                )),
                                asset_object::mime_type::set(Some("text/html".to_string())), // TODO maybe left empty is better
                            ],
                        )
                        .exec()
                        .await?
                }
            };

            let matches = client
                .file_path()
                .find_many(vec![
                    file_path::materialized_path::equals(materialized_path.into()),
                    file_path::name::starts_with(html_title.as_str().into()),
                ])
                .exec()
                .await?;
            let max_num = matches
                .iter()
                .filter_map(|file_path_data| {
                    let name = file_path_data.name.as_str();
                    if name == html_title.as_str() {
                        return Some(0);
                    }
                    let (name_1, num) = match name.rsplit_once(' ') {
                        Some((prefix, num)) => (prefix, num),
                        None => (name, "0"),
                    };
                    if name_1 == html_title.as_str() {
                        num.parse::<u32>().ok() // Converts from Result<T, E> to Option<T>
                    } else {
                        None
                    }
                })
                .max();

            let new_name = match max_num {
                Some(max_num) => format!("{} {}", html_title, max_num + 1),
                _ => html_title,
            };

            let valid_path_name = validators::replace_invalid_chars_in_path_name(new_name.as_str());
            let file_path_data = client
                .file_path()
                .create(
                    false,
                    materialized_path.to_string(),
                    valid_path_name,
                    vec![file_path::asset_object_id::set(Some(asset_object_data.id))],
                )
                .exec()
                .await?;
            Ok((asset_object_data, file_path_data, asset_object_existed))
        })
        .await
        .map_err(|e: QueryError| {
            rspc::Error::new(
                rspc::ErrorCode::InternalServerError,
                format!("failed to create asset_object: {}", e),
            )
        })?;

    Ok((file_path_data, asset_object_data, asset_object_existed))
}
