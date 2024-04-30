pub mod create;
mod delete;
mod process;
mod read;
mod types;
mod update;
mod utils;

use crate::routes::assets::update::update_doc_new_file;
use crate::routes::assets::update::update_file_path_and_doc;
use crate::routes::assets::update::update_folder_doc;
use crate::validators;
use crate::CtxWithLibrary;
use create::{create_asset_object, create_dir};
use delete::delete_file_path;
use p2p::PubsubMessage;
use prisma_lib::file_path;
use prisma_lib::read_filters::IntNullableFilter;
use process::{export_video_segment, process_video_asset, process_video_metadata};
use read::{get_file_path, list_file_path};
use rspc::{Router, RouterBuilder};
use serde::Deserialize;
use specta::Type;
use tracing::info;
use types::FilePathRequestPayload;
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
                    let (file_path_data, asset_object_data, asset_object_existed) =
                        create_asset_object(
                            &library,
                            &input.materialized_path,
                            &input.name,
                            &input.local_full_path,
                        )
                        .await?;
                    if !asset_object_existed {
                        process_video_metadata(&library, asset_object_data.id).await?;
                        info!("process video metadata finished");
                        process_video_asset(&library, &ctx, file_path_data.id, None).await?;
                        info!("process video asset finished");
                    }
                    let file_path = get_file_path(
                        &library,
                        &file_path_data.materialized_path,
                        &file_path_data.name,
                    )
                    .await?;
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
                    name: String,
                }
                |ctx, input: AssetObjectReceivePayload| async move {
                    tracing::debug!("received receive_asset: {input:?}");

                    let library = ctx.library()?;
                    let (file_path_data, asset_object_data, asset_object_existed) =
                        create_asset_object(
                            &library,
                            &input.materialized_path,
                            &input.name,
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
                        process_video_metadata(&library, asset_object_data.id).await?;
                        info!("process video metadata finished");
                        process_video_asset(&library, &ctx, file_path_data.id, Some(true)).await?;
                    }

                    Ok(())
                }
            })
        })
        .mutation("update_file_and_doc", |t| {
            t({
                #[derive(Deserialize, Type, Debug)]
                #[serde(rename_all = "camelCase")]
                struct AssetUpdateFileAndDocPayload {
                    path: String,
                    doc_id: String,
                    name: String,
                }
                |ctx, input: AssetUpdateFileAndDocPayload| async move {
                    tracing::debug!("update_file_and_doc: {input:?}");
                    let library = ctx.library()?;
                    update_file_path_and_doc(&library, input.path, input.name, input.doc_id)
                        .await?;
                    Ok(())
                }
            })
        })
        .mutation("update_folder_doc", |t| {
            t({
                #[derive(Deserialize, Type, Debug)]
                #[serde(rename_all = "camelCase")]
                struct AssetUpdateFolderDocPayload {
                    path: String,
                    doc_id: String,
                    name: String,
                }
                |ctx, input: AssetUpdateFolderDocPayload| async move {
                    tracing::debug!("update_folder_doc: {input:?}");
                    let library = ctx.library()?;
                    update_folder_doc(&library, input.path, input.name, input.doc_id.clone())
                        .await?;
                    // 再触发一次同步
                    if let Some(broadcast) = library.get_broadcast() {
                        let _ = broadcast
                            .send(PubsubMessage::Sync(input.doc_id))
                            .await
                            .unwrap();
                    }
                    Ok(())
                }
            })
        })
        // 用于文件任务完成，更新父级文件夹文档数据
        .mutation("update_doc_new_file", |t| {
            t({
                #[derive(Deserialize, Type, Debug)]
                #[serde(rename_all = "camelCase")]
                struct AssetUpdateDocNewFilePayload {
                    asset_object_id: i32,
                }
                |ctx, input: AssetUpdateDocNewFilePayload| async move {
                    tracing::debug!("update_doc_new_file: {input:?}");
                    let library = ctx.library()?;
                    let file_path_list = library
                        .prisma_client()
                        .file_path()
                        .find_many(vec![file_path::WhereParam::AssetObjectId(
                            IntNullableFilter::Equals(Some(input.asset_object_id)),
                        )])
                        .exec()
                        .await?;

                    for file_path in file_path_list {
                        let _ = update_doc_new_file(
                            &library,
                            file_path.materialized_path,
                            file_path.name,
                        )
                        .await?;
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
                    let item =
                        get_file_path(&library, &input.materialized_path, &input.name).await?;
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
                    // let broadcast = ctx.get_broadcast();
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
                process_video_asset(&library, &ctx, file_path_id, None).await?;
                Ok(())
            })
        })
        .mutation("process_video_metadata", |t| {
            t(|ctx, input: i32| async move {
                let library = ctx.library()?;
                let asset_object_id = input;
                process_video_metadata(&library, asset_object_id).await?;
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
                ).await?;
                Ok(())
            })
        })
}
