mod artifacts;
mod create;
mod delete;
mod read;
mod update;
mod utils;
mod web_page;

pub(crate) mod process;
pub(super) mod types;

use self::{
    create::{create_asset_object, create_dir},
    delete::delete_file_path,
    process::{build_content_index, export_video_segment, process_asset_metadata},
    read::{get_file_path, list_file_path},
    types::{FilePathRequestPayload, FilePathWithAssetObjectData},
    update::{move_file_path, rename_file_path},
    web_page::process_web_page,
};
use crate::{validators, CtxWithLibrary};
use rspc::{Router, RouterBuilder};
use serde::Deserialize;
use specta::Type;

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
                    // #[serde(deserialize_with = "validators::path_name_string")]
                    name: String,
                }
                |ctx, input: FilePathCreatePayload| async move {
                    let materialized_path = input.materialized_path;
                    let name = validators::replace_invalid_chars_in_path_name(input.name);
                    let library = ctx.library()?;
                    let file_path = create_dir(&library, &materialized_path, &name).await?;
                    Ok(file_path)
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
                    // #[serde(deserialize_with = "validators::path_name_string")]
                    name: String,
                    local_full_path: String,
                    // TODO: 加一个参数，指定是否需要删除源文件，对于客户端临时上传的文件，可以考虑删除
                }
                |ctx: TCtx, input: AssetObjectCreatePayload| async move {
                    tracing::info!("received create_asset_object: {input:?}");
                    let materialized_path = input.materialized_path;
                    let name = validators::replace_invalid_chars_in_path_name(input.name);
                    let library = ctx.library()?;
                    let content_base = ctx.content_base()?;
                    let (file_path_data, asset_object_data, asset_object_existed) =
                        create_asset_object(
                            &library,
                            &materialized_path,
                            &name,
                            &input.local_full_path,
                        )
                        .await?;
                    if !asset_object_existed {
                        process_asset_metadata(&library, &content_base, &asset_object_data.hash)
                            .await?;
                        tracing::info!("process metadata finished");
                        build_content_index(&library, &ctx, &asset_object_data.hash, false).await?;
                        tracing::info!("build content index finished");
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
                                .file_full_path_on_disk(&input.hash)
                                .to_string_lossy()
                                .to_string()
                                .as_str(),
                        )
                        .await?;

                    if asset_object_existed {
                        // TODO add artifacts merging logic
                    } else {
                        process_asset_metadata(&library, &content_base, &asset_object_data.hash)
                            .await?;
                        tracing::info!("process asset metadata finished");
                        build_content_index(&library, &ctx, &asset_object_data.hash, true).await?;
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
        .mutation("rebuild_content_index", |t| {
            #[derive(Deserialize, Type, Debug)]
            #[serde(rename_all = "camelCase")]
            struct RebuildIndexRequestPayload {
                asset_object_hash: String,
                with_existing_artifacts: bool,
            }
            t(|ctx: TCtx, input: RebuildIndexRequestPayload| async move {
                let library = ctx.library()?;
                build_content_index(
                    &library,
                    &ctx,
                    &input.asset_object_hash,
                    input.with_existing_artifacts,
                )
                .await?;
                Ok(())
            })
        })
        .mutation("process_asset_metadata", |t| {
            t(|ctx, input: String| async move {
                let library = ctx.library()?;
                let content_base = ctx.content_base()?;
                let asset_object_hash = input;
                process_asset_metadata(&library, &content_base, &asset_object_hash).await?;
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
                        build_content_index(&library, &ctx, asset_object_data.hash.as_str(), false)
                            .await?;
                        tracing::info!("process asset finished");
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
