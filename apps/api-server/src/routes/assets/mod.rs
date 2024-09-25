mod artifacts;
mod create;
mod delete;
pub(crate) mod process;
mod read;
pub mod types;
mod update;
mod utils;
mod web_page;

use crate::validators;
use crate::CtxWithLibrary;
use create::{create_asset_object, create_dir};
pub use delete::delete_file_path;
use process::export_video_segment;
use process::process_asset;
use process::process_asset_metadata;
use read::{get_file_path, list_file_path};
use rspc::{Router, RouterBuilder};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::path::PathBuf;
use tokio::{fs::OpenOptions, io::AsyncWriteExt};
use tracing::info;
use types::FilePathRequestPayload;
use types::FilePathWithAssetObjectData;
use update::{move_file_path, rename_file_path};
use web_page::process_web_page;

pub fn get_routes<TCtx>() -> RouterBuilder<TCtx>
where
    TCtx: CtxWithLibrary + Clone + Send + Sync + 'static,
{
    Router::<TCtx>::new()
        .mutation("create_dir", |t| {
            t({
                #[derive(Deserialize, Type, Debug)]
                #[serde(rename_all = "camelCase")]
                struct FilePathCreatePayload {
                    #[serde(deserialize_with = "validators::materialized_path_string")]
                    materialized_path: String,
                    #[serde(deserialize_with = "validators::path_name_string")]
                    name: String,
                }
                |ctx, input: FilePathCreatePayload| async move {
                    let library = ctx.library()?;
                    create_dir(&library, &input.materialized_path, &input.name).await?;
                    Ok(())
                }
            })
        })
        .mutation("create_asset_object", |t| {
            t({
                #[derive(Deserialize, Type, Debug)]
                #[serde(rename_all = "camelCase")]
                struct AssetObjectCreatePayload {
                    #[serde(deserialize_with = "validators::materialized_path_string")]
                    materialized_path: String,
                    name: String,
                    local_full_path: String,
                    // TODO: 加一个参数，指定是否需要删除源文件，对于客户端临时上传的文件，可以考虑删除
                }
                |ctx: TCtx, input: AssetObjectCreatePayload| async move {
                    info!("received create_asset_object: {input:?}");
                    let library = ctx.library()?;
                    let content_base = ctx.content_base()?;
                    let (file_path_data, asset_object_data, asset_object_existed) =
                        create_asset_object(
                            &library,
                            &input.materialized_path,
                            &input.name,
                            &input.local_full_path,
                        )
                        .await?;
                    if !asset_object_existed {
                        process_asset_metadata(
                            &library,
                            &content_base,
                            asset_object_data.id,
                            Some(&input.local_full_path),
                        )
                        .await?;
                        info!("process metadata finished");
                        process_asset(&library, &ctx, asset_object_data.hash, None).await?;
                        info!("process asset finished");
                    }
                    let file_path: FilePathWithAssetObjectData = get_file_path(
                        &library,
                        &file_path_data.materialized_path,
                        &file_path_data.name,
                    )
                    .await?
                    .into();
                    Ok(file_path)
                }
            })
        })
        .mutation("receive_asset", |t| {
            t({
                #[derive(Deserialize, Type, Debug)]
                #[serde(rename_all = "camelCase")]
                struct AssetObjectReceivePayload {
                    hash: String,
                    #[serde(deserialize_with = "validators::materialized_path_string")]
                    materialized_path: String,
                }
                |ctx, input: AssetObjectReceivePayload| async move {
                    tracing::debug!("received receive_asset: {input:?}");

                    let library = ctx.library()?;
                    let content_base = ctx.content_base()?;
                    let (_file_path_data, asset_object_data, asset_object_existed) =
                        create_asset_object(
                            &library,
                            &input.materialized_path,
                            &input.hash,
                            &library
                                .file_path(&input.hash)
                                .to_string_lossy()
                                .to_string()
                                .as_str(),
                        )
                        .await?;

                    if asset_object_existed {
                        // TODO add artifacts merging logic
                    } else {
                        process_asset_metadata(
                            &library,
                            &content_base,
                            asset_object_data.id,
                            Some(
                                &library
                                    .file_path(&input.hash)
                                    .to_string_lossy()
                                    .to_string()
                                    .as_str(),
                            ),
                        )
                        .await?;
                        info!("process asset metadata finished");
                        process_asset(&library, &ctx, asset_object_data.hash, Some(true)).await?;
                    }

                    Ok(())
                }
            })
        })
        .mutation("upload_file_chunk_to_temp", |t| {
            // TODO: 使用 OpenDAL 来实现
            // 注意: 这个接口只能一个个调用，不然上传同一个文件会出现并发写的问题
            t({
                #[derive(Serialize, Deserialize, Type, Debug)]
                #[serde(rename_all = "camelCase")]
                struct FileChunkUploadResult {
                    full_path: String, // whether the file is fully uploaded
                    chunk_index: u32,
                    message: String,
                }
                #[derive(Deserialize, Type, Debug)]
                #[serde(rename_all = "camelCase")]
                struct FileChunkUploadData {
                    file_name: String,
                    chunk_index: u32,
                    total_chunks: u32,
                    chunk: Vec<u8>,
                }
                |_ctx, chunk_data: FileChunkUploadData| async move {
                    let temp_dir_root = std::env::temp_dir();
                    let full_path = {
                        let temp_dir = temp_dir_root.join("gendam-file-upload");
                        std::fs::create_dir_all(&temp_dir).map_err(|e| {
                            rspc::Error::new(
                                rspc::ErrorCode::InternalServerError,
                                format!("Failed to create temporary directory: {}", e),
                            )
                        })?;
                        temp_dir.join(&chunk_data.file_name)
                    };

                    if chunk_data.chunk_index == 0 && full_path.exists() {
                        std::fs::remove_file(&full_path).map_err(|e| {
                            rspc::Error::new(
                                rspc::ErrorCode::InternalServerError,
                                format!("Failed to delete existing file: {}", e),
                            )
                        })?;
                    }

                    let mut file = OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(&full_path)
                        .await
                        .map_err(|e| {
                            rspc::Error::new(
                                rspc::ErrorCode::InternalServerError,
                                format!("Failed to open file: {}", e),
                            )
                        })?;

                    file.write_all(&chunk_data.chunk).await.map_err(|e| {
                        rspc::Error::new(
                            rspc::ErrorCode::InternalServerError,
                            format!("Failed to write chunk: {}", e),
                        )
                    })?;

                    if chunk_data.chunk_index == chunk_data.total_chunks - 1 {
                        // 最后一个分片，可以进行文件完整性检查等操作
                    }

                    Ok(FileChunkUploadResult {
                        full_path: full_path.to_string_lossy().to_string(),
                        chunk_index: chunk_data.chunk_index,
                        message: format!("Chunk {} uploaded successfully", chunk_data.chunk_index),
                    })
                }
            })
        })
        .query("list", |t| {
            t({
                #[derive(Deserialize, Type, Debug)]
                #[serde(rename_all = "camelCase")]
                struct FilePathQueryPayload {
                    // #[serde(rename = "materializedPath")]
                    #[serde(deserialize_with = "validators::materialized_path_string")]
                    materialized_path: String,
                    // export `isDir?: boolean` instead of `isDir: boolean | null`
                    #[serde(skip_serializing_if = "Option::is_none")]
                    is_dir: Option<bool>,
                    #[serde(skip_serializing_if = "Option::is_none")]
                    include_sub_dirs: Option<bool>,
                }
                |ctx, input: FilePathQueryPayload| async move {
                    let library = ctx.library()?;
                    let res = list_file_path(
                        &library,
                        &input.materialized_path,
                        input.is_dir,
                        input.include_sub_dirs,
                    )
                    .await?;

                    let res: Vec<FilePathWithAssetObjectData> =
                        res.into_iter().map(|v| v.into()).collect();

                    Ok(res)
                }
            })
        })
        .query("get", |t| {
            t({
                #[derive(Deserialize, Type, Debug)]
                #[serde(rename_all = "camelCase")]
                struct FilePathGetPayload {
                    #[serde(deserialize_with = "validators::materialized_path_string")]
                    materialized_path: String,
                    #[serde(deserialize_with = "validators::path_name_string")]
                    name: String,
                }
                |ctx, input: FilePathGetPayload| async move {
                    let library = ctx.library()?;
                    let item: FilePathWithAssetObjectData =
                        get_file_path(&library, &input.materialized_path, &input.name)
                            .await?
                            .into();
                    Ok(item)
                }
            })
        })
        .mutation("rename_file_path", |t| {
            t({
                #[derive(Deserialize, Type, Debug)]
                #[serde(rename_all = "camelCase")]
                struct FilePathRenamePayload {
                    id: i32,
                    is_dir: bool,
                    #[serde(deserialize_with = "validators::materialized_path_string")]
                    materialized_path: String,
                    #[serde(deserialize_with = "validators::path_name_string")]
                    old_name: String,
                    #[serde(deserialize_with = "validators::path_name_string")]
                    new_name: String,
                }
                |ctx, input: FilePathRenamePayload| async move {
                    let library = ctx.library()?;
                    rename_file_path(
                        &library,
                        input.id,
                        input.is_dir,
                        &input.materialized_path,
                        &input.old_name,
                        &input.new_name,
                    )
                    .await?;
                    Ok(())
                }
            })
        })
        .mutation("move_file_path", |t| {
            t({
                #[derive(Deserialize, Type, Debug)]
                #[serde(rename_all = "camelCase")]
                struct FilePathMovePayload {
                    active: FilePathRequestPayload,
                    target: Option<FilePathRequestPayload>,
                }
                |ctx, input: FilePathMovePayload| async move {
                    let library = ctx.library()?;
                    move_file_path(&library, input.active, input.target).await?;
                    Ok(())
                }
            })
        })
        .mutation("delete_file_path", |t| {
            t({
                #[derive(Deserialize, Type, Debug)]
                #[serde(rename_all = "camelCase")]
                struct FilePathDeletePayload {
                    #[serde(deserialize_with = "validators::materialized_path_string")]
                    materialized_path: String,
                    #[serde(deserialize_with = "validators::path_name_string")]
                    name: String,
                }
                |ctx, input: FilePathDeletePayload| async move {
                    delete_file_path(&ctx, &input.materialized_path, &input.name).await?;
                    Ok(())
                }
            })
        })
        .mutation("process_asset", |t| {
            t(|ctx, input: String| async move {
                let library = ctx.library()?;
                let asset_object_hash = input;
                process_asset(&library, &ctx, asset_object_hash, None).await?;
                Ok(())
            })
        })
        .mutation("process_asset_metadata", |t| {
            t(|ctx, input: i32| async move {
                let library = ctx.library()?;
                let content_base = ctx.content_base()?;
                let asset_object_id = input;
                process_asset_metadata(
                    &library,
                    &content_base,
                    asset_object_id,
                    Option::<PathBuf>::None,
                )
                .await?;
                Ok(())
            })
        })
        .mutation("export_video_segment", |t| {
            #[derive(Deserialize, Type, Debug)]
            #[serde(rename_all = "camelCase")]
            struct VideoSegmentExportPayload {
                verbose_file_name: String,
                output_dir: String,
                asset_object_id: i32,
                milliseconds_from: u32,
                milliseconds_to: u32,
            }
            t(|ctx, input: VideoSegmentExportPayload| async move {
                let library = ctx.library()?;
                export_video_segment(
                    &library,
                    input.verbose_file_name,
                    input.output_dir,
                    input.asset_object_id,
                    input.milliseconds_from,
                    input.milliseconds_to,
                )
                .await?;
                Ok(())
            })
        })
        .mutation("create_web_page_object", |t| {
            #[derive(Deserialize, Type, Debug)]
            #[serde(rename_all = "camelCase")]
            struct WebPageCreatePayload {
                #[serde(deserialize_with = "validators::materialized_path_string")]
                materialized_path: String,
                url: String,
            }
            t({
                |ctx: TCtx, payload: WebPageCreatePayload| async move {
                    tracing::debug!("create_web_page_object: {:?}", payload);
                    let library = ctx.library()?;

                    let (file_path_data, asset_object_data, asset_object_existed) =
                        process_web_page(&library, &payload.materialized_path, &payload.url)
                            .await?;
                    if !asset_object_existed {
                        process_asset(&library, &ctx, asset_object_data.hash, None).await?;
                        info!("process asset finished");
                    }
                    let file_path: FilePathWithAssetObjectData = get_file_path(
                        &library,
                        &file_path_data.materialized_path,
                        &file_path_data.name,
                    )
                    .await?
                    .into();
                    Ok(file_path)
                }
            })
        })
        .merge("artifacts.", artifacts::get_routes::<TCtx>())
}
