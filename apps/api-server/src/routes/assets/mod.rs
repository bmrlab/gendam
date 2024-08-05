mod create;
mod delete;
mod process;
mod read;
pub mod types;
mod update;
mod utils;

use crate::validators;
use crate::CtxWithLibrary;
use create::{create_asset_object, create_dir};
pub use delete::delete_file_path;
use process::export_video_segment;
use process::process_asset;
use process::process_asset_metadata;
use read::{get_file_path, list_file_path};
use rspc::{Router, RouterBuilder};
use serde::Deserialize;
use specta::Type;
use tracing::info;
use types::FilePathRequestPayload;
use types::FilePathWithAssetObjectData;
use update::{move_file_path, rename_file_path};

pub fn get_routes<TCtx>() -> RouterBuilder<TCtx>
where
    TCtx: CtxWithLibrary + Clone + Send + Sync + 'static,
{
    #[derive(Deserialize, Type, Debug)]
    #[serde(rename_all = "camelCase")]
    struct AssetObjectCreatePayload {
        #[serde(deserialize_with = "validators::materialized_path_string")]
        materialized_path: String,
        name: String,
        local_full_path: String,
    }
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
                        process_asset_metadata(&library, &content_base, asset_object_data.id)
                            .await?;
                        info!("process metadata finished");
                        process_asset(&library, &ctx, file_path_data.id, None).await?;
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
                    let (file_path_data, asset_object_data, asset_object_existed) =
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
                        process_asset_metadata(&library, &content_base, asset_object_data.id)
                            .await?;
                        info!("process video metadata finished");
                        process_asset(&library, &ctx, file_path_data.id, Some(true)).await?;
                    }

                    Ok(())
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
        .mutation("process_video_asset", |t| {
            t(|ctx, input: i32| async move {
                let library = ctx.library()?;
                let file_path_id = input;
                process_asset(&library, &ctx, file_path_id, None).await?;
                Ok(())
            })
        })
        .mutation("process_video_metadata", |t| {
            t(|ctx, input: i32| async move {
                let library = ctx.library()?;
                let content_base = ctx.content_base()?;
                let asset_object_id = input;
                process_asset_metadata(&library, &content_base, asset_object_id).await?;
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
}
