mod utils;
mod types;
mod create;
mod process;
mod read;
mod update;
mod delete;

use rspc::{Router, Rspc};
use serde::Deserialize;
use specta::Type;
use crate::CtxWithLibrary;

use create::{create_file_path, create_asset_object};
use process::{process_video_asset, process_video_metadata};
use read::{get_file_path, list_file_path};
use update::{move_file_path, rename_file_path};
use delete::delete_file_path;
use types::FilePathRequestPayload;


pub fn get_routes<TCtx>() -> Router<TCtx>
where
    TCtx: CtxWithLibrary + Clone + Send + Sync + 'static,
{
    let router = Rspc::<TCtx>::new()
        .router()
        .procedure(
            "create_file_path",
            Rspc::<TCtx>::new().mutation({
                #[derive(Deserialize, Type, Debug)]
                struct FilePathCreatePayload {
                    path: String,
                    name: String,
                }
                |ctx, input: FilePathCreatePayload| async move {
                    let library = ctx.library()?;
                    create_file_path(&library, &input.path, &input.name).await?;
                    Ok(())
                }
            }),
        )
        .procedure(
            "create_asset_object",
            Rspc::<TCtx>::new().mutation({
                #[derive(Deserialize, Type, Debug)]
                #[serde(rename_all = "camelCase")]
                struct AssetObjectCreatePayload {
                    path: String,
                    local_full_path: String,
                }
                |ctx: TCtx, input: AssetObjectCreatePayload| async move {
                    let library = ctx.library()?;
                    let (file_path_data, asset_object_data) =
                        create_asset_object(&library, &input.path, &input.local_full_path)
                        .await?;
                    process_video_asset(&library, &ctx, file_path_data.id).await?;
                    process_video_metadata(&library, &ctx, asset_object_data.id).await?;
                    Ok(())
                }
            })
        )
        .procedure(
            "list",
            Rspc::<TCtx>::new().query({
                #[derive(Deserialize, Type, Debug)]
                struct FilePathQueryPayload {
                    path: String,
                    #[serde(rename = "dirsOnly")]
                    dirs_only: bool,
                }
                |ctx, input: FilePathQueryPayload| async move {
                    let library = ctx.library()?;
                    let names =
                        list_file_path(&library, &input.path, input.dirs_only)
                        .await?;
                    Ok(names)
                }
            })
        )
        .procedure(
            "get",
            Rspc::<TCtx>::new().query({
                #[derive(Deserialize, Type, Debug)]
                #[serde(rename_all = "camelCase")]
                struct FilePathGetPayload {
                    path: String,
                    name: String,
                }
                |ctx, input: FilePathGetPayload| async move {
                    let library = ctx.library()?;
                    let item = get_file_path(&library, &input.path, &input.name).await?;
                    Ok(item)
                }
            })
        )
        .procedure(
            "rename_file_path",
            Rspc::<TCtx>::new().mutation({
                #[derive(Deserialize, Type, Debug)]
                #[serde(rename_all = "camelCase")]
                struct FilePathRenamePayload {
                    id: i32,
                    is_dir: bool,
                    path: String,
                    old_name: String,
                    new_name: String,
                }
                |ctx, input: FilePathRenamePayload| async move {
                    let library = ctx.library()?;
                    rename_file_path(
                        &library, input.id, input.is_dir,
                        &input.path, &input.old_name, &input.new_name
                    ).await?;
                    Ok(())
                }
            })
        )
        .procedure(
            "move_file_path",
            Rspc::<TCtx>::new().mutation({
                #[derive(Deserialize, Type, Debug)]
                #[serde(rename_all = "camelCase")]
                struct FilePathMovePayload {
                    active: FilePathRequestPayload,
                    target: Option<FilePathRequestPayload>,
                }
                |ctx, input: FilePathMovePayload| async move {
                    let library = ctx.library()?;
                    move_file_path(
                        &library, input.active, input.target
                    ).await?;
                    Ok(())
                }
            })
        )
        .procedure(
            "delete_file_path",
            Rspc::<TCtx>::new().mutation({
                #[derive(Deserialize, Type, Debug)]
                #[serde(rename_all = "camelCase")]
                struct FilePathDeletePayload {
                    path: String,
                    name: String,
                }
                |ctx, input: FilePathDeletePayload| async move {
                    let library = ctx.library()?;
                    delete_file_path(&library, &input.path, &input.name).await?;
                    Ok(())
                }
            })
        )
        .procedure(
            "process_video_asset",
            Rspc::<TCtx>::new().mutation(|ctx, input: i32| async move {
                let library = ctx.library()?;
                let file_path_id = input;
                process_video_asset(&library, &ctx, file_path_id).await?;
                Ok(())
            })
        )
        .procedure(
            "process_video_metadata",
            Rspc::<TCtx>::new().mutation(|ctx, input: i32| async move {
                let library = ctx.library()?;
                let file_path_id = input;
                process_video_metadata(&library, &ctx, file_path_id).await?;
                Ok(())
            })
        );
    router
}
