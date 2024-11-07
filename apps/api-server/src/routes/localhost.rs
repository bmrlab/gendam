use super::super::ctx::traits::CtxWithLibrary;
use axum::{
    extract::{DefaultBodyLimit, Json as ExtractJson, Path as ExtractPath},
    http::StatusCode,
    response::{IntoResponse, Json as JsonResponse, Redirect},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::collections::HashMap;
use std::io::SeekFrom;
use std::sync::Arc;
use tokio::io::AsyncSeekExt;
use tokio::sync::Mutex;
use tokio::{fs::OpenOptions, io::AsyncWriteExt};
use tower_http::services::ServeDir;

// 全局文件锁管理器
struct UploadManager {
    file_locks: HashMap<String, Arc<Mutex<()>>>,
}

impl UploadManager {
    fn new() -> Self {
        Self {
            file_locks: HashMap::new(),
        }
    }

    fn get_lock(&mut self, filename: &str) -> Arc<Mutex<()>> {
        self.file_locks
            .entry(filename.to_string())
            .or_insert_with(|| Arc::new(Mutex::new(())))
            .clone()
    }
}

// 使用lazy_static来管理全局上传管理器
lazy_static::lazy_static! {
    static ref UPLOAD_MANAGER: Mutex<UploadManager> = Mutex::new(UploadManager::new());
}

pub fn get_routes<TCtx>(ctx: TCtx) -> Router
where
    TCtx: CtxWithLibrary + Clone + Send + Sync + 'static,
{
    Router::new()
        .route("/_storage/localhost/asset_object/:hash/file", {
            let ctx = ctx.clone();
            get(|ExtractPath(hash): ExtractPath<String>| async move {
                match ctx.library() {
                    Ok(library) => {
                        let file_full_path_on_disk = library.file_full_path_on_disk(hash.as_str());
                        let new_path = format!(
                            "/_unsafe/localhost{}",
                            file_full_path_on_disk.to_string_lossy().as_ref()
                        );
                        Ok(Redirect::permanent(&new_path))
                    }
                    Err(e) => {
                        tracing::error!("Failed to load library: {:?}", e);
                        Err("Failed to load library")
                    }
                }
            })
        })
        .route("/_storage/localhost/asset_object/:hash/artifacts/*rest", {
            // *rest will match the rest of the path including the slashes
            let ctx = ctx.clone();
            get(
                |ExtractPath((hash, rest)): ExtractPath<(String, String)>| async move {
                    match ctx.library() {
                        Ok(library) => {
                            let artifacts_dir_path_on_disk =
                                library._absolute_artifacts_dir(hash.as_str());
                            let file_full_path_on_disk = artifacts_dir_path_on_disk.join(rest);
                            let new_path = format!(
                                "/_unsafe/localhost{}",
                                file_full_path_on_disk.to_string_lossy().as_ref()
                            );
                            Ok(Redirect::permanent(&new_path))
                        }
                        Err(e) => {
                            tracing::error!("Failed to load library: {:?}", e);
                            Err("Failed to load library")
                        }
                    }
                },
            )
        })
        .route("/_storage/localhost/upload_file_chunk_to_temp/", {
            // let ctx = ctx.clone();
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
                chunk_size: u32,
                chunk_index: u32,
                total_chunks: u32,
                chunk: Vec<u8>,
            }
            post(
                |ExtractJson(chunk_data): ExtractJson<FileChunkUploadData>| async move {
                    // 获取文件锁
                    let file_lock = {
                        let mut manager = UPLOAD_MANAGER.lock().await;
                        manager.get_lock(&chunk_data.file_name)
                    };
                    // 获取文件锁，会在函数退出以后释放，千万别放进 {} block 中
                    let _guard = file_lock.lock().await;

                    // temp dir will be cleaned up by the OS on macOS
                    let temp_dir_root = std::env::temp_dir();
                    let full_path = {
                        let temp_dir = temp_dir_root.join("gendam-file-upload");
                        if let Err(e) = std::fs::create_dir_all(&temp_dir) {
                            return (
                                StatusCode::INTERNAL_SERVER_ERROR,
                                format!("Failed to create temporary directory: {}", e),
                            )
                                .into_response();
                        };
                        temp_dir.join(&chunk_data.file_name)
                    };

                    // 初始化文件，第 0 个 chunk 要单独，确保新建文件
                    if chunk_data.chunk_index == 0 && !full_path.exists() {
                        if let Err(e) = tokio::fs::File::create(&full_path).await {
                            return (
                                StatusCode::INTERNAL_SERVER_ERROR,
                                format!("Failed to create file: {}", e),
                            )
                                .into_response();
                        }
                    }

                    // 打开文件用于随机访问, 不要用 append
                    let mut file = match OpenOptions::new()
                        .read(true)
                        .write(true)
                        .open(&full_path)
                        .await
                    {
                        Ok(file) => file,
                        Err(e) => {
                            return (
                                StatusCode::INTERNAL_SERVER_ERROR,
                                format!("Failed to open file: {}", e),
                            )
                                .into_response();
                        }
                    };

                    let offset = chunk_data.chunk_index * chunk_data.chunk_size;
                    if let Err(e) = file.seek(SeekFrom::Start(offset as u64)).await {
                        return (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            format!("Failed to seek file: {}", e),
                        )
                            .into_response();
                    }

                    // 写入分片数据
                    if let Err(e) = file.write_all(&chunk_data.chunk).await {
                        return (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            format!("Failed to write chunk: {}", e),
                        )
                            .into_response();
                    }

                    // 确保数据写入磁盘
                    if let Err(e) = file.sync_all().await {
                        return (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            format!("Failed to sync file: {}", e),
                        )
                            .into_response();
                    }

                    if chunk_data.chunk_index == chunk_data.total_chunks - 1 {
                        // 最后一个分片，可以进行文件完整性检查等操作
                    }

                    let json_response = JsonResponse(FileChunkUploadResult {
                        full_path: full_path.to_string_lossy().to_string(),
                        chunk_index: chunk_data.chunk_index,
                        message: format!("Chunk {} uploaded successfully", chunk_data.chunk_index),
                    });

                    (StatusCode::OK, json_response).into_response()
                },
            )
            .layer(DefaultBodyLimit::max(10 * 1024 * 1024))
        })
        // .nest_service("/artifacts", ServeDir::new(local_data_dir.clone()))
        .nest_service("/_unsafe/localhost/", ServeDir::new("/"))
}
