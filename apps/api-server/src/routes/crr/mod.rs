mod association;

use crate::routes::crr::association::{get_ids_for_dir, get_ids_for_file};
use crate::validators;
use crate::CtxWithLibrary;
use crdt::sync::FileSync;
use crdt::CrsqlChangesRowData;
use prisma_lib::sync_metadata::sub_file_path_ids;
use prisma_lib::{file_path, sync_metadata};
use rspc::{Router, RouterBuilder};
use serde::{Deserialize, Serialize};
use specta::Type;
use tracing::{debug, info};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct PullResult {
    device_id: String,
    changes: Vec<CrsqlChangesRowData>,
    file_path_id: String,
    sub_file_path_ids: Vec<String>,
}

impl PullResult {
    pub fn new(
        changes: Vec<CrsqlChangesRowData>,
        file_path_id: String,
        sub_file_path_ids: Vec<String>,
    ) -> Self {
        Self {
            // TODO: replace with real value
            device_id: "".to_string(),
            changes,
            file_path_id,
            sub_file_path_ids,
        }
    }
}

pub fn get_routes<TCtx>() -> RouterBuilder<TCtx>
    where
        TCtx: CtxWithLibrary + Clone + Send + Sync + 'static,
{
    Router::<TCtx>::new()
        .mutation("pull", |t| {
            t(|ctx, file_path_id: String| async move {
                debug!("pull payload: {:?}", file_path_id);

                let library = ctx.library()?;

                let (file_path_id, asset_object_id, media_data_id) =
                    get_ids_for_file(library.prisma_client(), file_path_id.clone()).await?;

                debug!(
                    "file_path_id: {:?}, asset_object_id: {:?}, media_data_id: {:?}",
                    file_path_id, asset_object_id, media_data_id
                );

                let file_sync = FileSync::new(library.db_path());

                // TODO: replace db_version with real value
                let changes = file_sync
                    .pull_file_changes(0, asset_object_id, file_path_id.clone(), media_data_id)
                    .map_err(|e| {
                        rspc::Error::new(
                            rspc::ErrorCode::InternalServerError,
                            format!("failed to pull {file_path_id} changes: {e}"),
                        )
                    })?;

                // TODO: 序列化
                let pull_result = PullResult::new(changes, file_path_id, vec![]);
                debug!("pull result: {pull_result:?}");

                Ok(serde_json::to_string(&pull_result).map_err(|e| {
                    rspc::Error::new(
                        rspc::ErrorCode::InternalServerError,
                        format!("failed to serialize pull result: {e}"),
                    )
                })?)
            })
        })
        .mutation("pull_dir", |t| {
            #[derive(Deserialize, Type, Debug)]
            #[serde(rename_all = "camelCase")]
            struct PullDirPayload {
                dir: String,
                dir_file_path_id: String,
            }

            t(|ctx, payload: PullDirPayload| async move {
                debug!("pull_dir payload: {:?}", payload);
                let library = ctx.library()?;
                let mut ids = get_ids_for_dir(library.prisma_client(), payload.dir).await?;
                let sub_file_path_ids = ids.0.clone();
                // add dir file path id
                ids.0.push(payload.dir_file_path_id.clone());
                debug!("ids: {:?}", ids);

                let file_sync = FileSync::new(library.db_path());

                // TODO: replace db_version with real value
                let changes = file_sync
                    .pull_dir_changes(0, ids.0, ids.1, ids.2)
                    .map_err(|e| {
                        rspc::Error::new(
                            rspc::ErrorCode::InternalServerError,
                            format!("failed to pull {} changes: {e}", payload.dir_file_path_id),
                        )
                    })?;

                let pull_result =
                    PullResult::new(changes, payload.dir_file_path_id, sub_file_path_ids);

                Ok(serde_json::to_string(&pull_result).map_err(|e| {
                    rspc::Error::new(
                        rspc::ErrorCode::InternalServerError,
                        format!("failed to serialize pull dir result: {e}"),
                    )
                })?)
            })
        })
        .mutation("apply", |t| {
            #[derive(Deserialize, Type, Debug, Clone)]
            #[serde(rename_all = "camelCase")]
            struct ApplyPayload {
                /// 这个就是 materialized_path 字段的值
                #[serde(deserialize_with = "validators::materialized_path_string")]
                relative_path: String,
                pull_result: String,
            }
            t(|ctx, payload: ApplyPayload| async move {
                info!("api changes: {:?}", payload);
                let library = ctx.library()?;

                let pull_result: PullResult =
                    serde_json::from_str(&payload.pull_result).map_err(|e| {
                        rspc::Error::new(
                            rspc::ErrorCode::InternalServerError,
                            format!("failed to deserialize pull result: {e}"),
                        )
                    })?;

                let mut file_sync = FileSync::new(library.db_path());
                file_sync
                    .apple_changes(pull_result.changes)
                    .expect("Failed to apply changes");

                let sub_file_path_ids_string =
                    serde_json::to_string(&pull_result.sub_file_path_ids).ok();

                // 添加 sync metadata 信息
                library
                    .prisma_client()
                    .sync_metadata()
                    .upsert(
                        sync_metadata::UniqueWhereParam::FilePathIdEquals(
                            pull_result.file_path_id.clone(),
                        ),
                        (
                            file_path::UniqueWhereParam::IdEquals(pull_result.file_path_id.clone()),
                            pull_result.device_id.clone(),
                            payload.relative_path.clone(),
                            vec![sub_file_path_ids::set(sub_file_path_ids_string.clone())],
                        ),
                        vec![
                            sync_metadata::device_id::set(pull_result.device_id),
                            sync_metadata::relative_path::set(payload.relative_path.clone()),
                            sub_file_path_ids::set(sub_file_path_ids_string),
                        ],
                    )
                    .exec()
                    .await?;

                // 为同步的 file_path 添加 relative path
                // 以便日后识别
                // 如果不方便加，可以在 routes/assets/utils 中加以查询
                library
                    .prisma_client()
                    .file_path()
                    .update_many(
                        vec![file_path::WhereParam::Id(
                            prisma_lib::read_filters::StringFilter::InVec(
                                [
                                    pull_result.sub_file_path_ids,
                                    vec![pull_result.file_path_id],
                                ]
                                    .concat(),
                            ),
                        )],
                        vec![file_path::SetParam::SetRelativePath(Some(
                            payload.relative_path,
                        ))],
                    )
                    .exec()
                    .await?;

                Ok(())
            })
        })
}
