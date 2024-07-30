mod rag;
mod recommend;
mod search;

use crate::{get_hash_from_url, get_library_settings, CtxWithLibrary};
use glob::glob;
use rag::{rag_with_video, RAGRequestPayload};
use recommend::{recommend_frames, RecommendRequestPayload};
use rspc::{Router, RouterBuilder};
use search::{search_all, SearchRequestPayload};
use storage::Storage;
use storage::{EntryMode, S3Storage};
use tokio::sync::mpsc;

pub fn get_routes<TCtx>() -> RouterBuilder<TCtx>
where
    TCtx: CtxWithLibrary + Clone + Send + Sync + 'static,
{
    Router::<TCtx>::new()
        .query("all", |t| {
            t(|ctx: TCtx, input: SearchRequestPayload| async move {
                let library = ctx.library()?;
                let content_base = ctx.content_base()?;
                search_all(&library, &content_base, input).await
            })
        })
        .query("recommend", |t| {
            t(|ctx: TCtx, input: RecommendRequestPayload| async move {
                let library = ctx.library()?;
                let content_base = ctx.content_base()?;
                recommend_frames(
                    &library,
                    &content_base,
                    &input.asset_object_hash,
                    input.timestamp,
                )
                .await
            })
        })
        .query("suggestions", |t| {
            t(|ctx: TCtx, _input: ()| async move {
                let library = ctx.library()?;
                // let asset_object_data_list = library
                //     .prisma_client()
                //     .asset_object()
                //     .find_many(vec![])
                //     .exec()
                //     .await
                //     .map_err(sql_error)?;
                // let captions = asset_object_data_list
                //     .into_iter()
                //     .filter_map(|asset_object_data| {
                //         // let video_handler = VideoHandler::new(
                //         //     &asset_object_data.hash, &library
                //         // ).ok()?;
                //         // video_handler.get_artifacts_settings().ok()?;
                //         Some("".to_string())
                //     })
                //     .collect::<Vec<String>>();
                // Search local
                let pattern = format!(
                    "{}/artifacts/*/*/frame-caption-*/*.json",
                    library.dir.to_string_lossy()
                );
                let entries = glob(&pattern).map_err(|e| {
                    rspc::Error::new(
                        rspc::ErrorCode::InternalServerError,
                        format!("glob failed: {}", e),
                    )
                })?;
                let mut already_seen = std::collections::HashSet::new();
                let mut results = entries
                    .into_iter()
                    .filter_map(|entry| {
                        let json_path = entry.ok()?;
                        dbg!(&json_path);
                        dbg!(get_hash_from_url(json_path.as_os_str().to_str()?));
                        if let Some(hash) = get_hash_from_url(json_path.as_os_str().to_str()?) {
                            already_seen.insert(hash);
                        }
                        let json_str = std::fs::read_to_string(&json_path).ok()?;
                        let json_val = serde_json::from_str::<serde_json::Value>(&json_str).ok()?;
                        let caption = json_val.get("caption")?.as_str()?;
                        Some(caption.to_owned())
                    })
                    .collect::<Vec<String>>();
                // Search S3
                // glob not support now https://github.com/apache/opendal/issues/1251
                if let Ok(s3_config) = get_library_settings(&library.dir).s3_config.ok_or(()) {
                    if let Ok(storage) = S3Storage::new(&library.id, s3_config) {
                        if let Ok(op) = storage.op() {
                            match op.list_with("artifacts").recursive(true).await {
                                Ok(entries) => {
                                    let mut s3_results = Vec::new();
                                    for entry in entries {
                                        if !entry.path().ends_with(".json")
                                            || !(entry.path().contains("frame-caption-")
                                                && !entry
                                                    .path()
                                                    .contains("frame-caption-embedding-"))
                                        {
                                            continue;
                                        }

                                        if let Some(hash) = get_hash_from_url(entry.path()) {
                                            if already_seen.contains(&hash) {
                                                continue;
                                            }
                                        }

                                        match entry.metadata().mode() {
                                            EntryMode::FILE => match op.read(entry.path()).await {
                                                Ok(data) => {
                                                    if let Ok(data) =
                                                        String::from_utf8(data.to_vec())
                                                    {
                                                        serde_json::from_str::<serde_json::Value>(
                                                            &data,
                                                        )
                                                        .ok()
                                                        .map(|value| {
                                                            value.get("caption").map(|v| {
                                                                v.as_str().map(|s| {
                                                                    s3_results.push(s.to_owned());
                                                                })
                                                            });
                                                        });
                                                    }
                                                }
                                                Err(e) => {
                                                    tracing::error!("failed to read file: {}", e);
                                                }
                                            },
                                            _ => continue,
                                        }
                                    }
                                    results.extend(s3_results);
                                }
                                Err(e) => {
                                    tracing::error!("failed to list s3 entries: {}", e);
                                }
                            }
                        }
                    }
                }
                Ok(results)
            })
        })
        .subscription("video_rag", |t| {
            t(|ctx, input: RAGRequestPayload| {
                tracing::debug!("receive chat request");

                let library = ctx.library().expect("library is valid");
                let content_base = ctx.content_base().expect("content base is valid");
                let ai_handler = ctx.ai_handler().expect("ai handler is valid");

                return async_stream::stream! {
                    let (tx, mut rx) = mpsc::channel(512);

                    tokio::spawn(async move {
                        if let Err(e) = rag_with_video(&library, &content_base, &ai_handler, input, tx).await {
                            tracing::error!("RAG error: {}", e);
                        }
                    });

                    while let Some(event) = rx.recv().await {
                        yield event;
                    }
                };
            })
        })
}
