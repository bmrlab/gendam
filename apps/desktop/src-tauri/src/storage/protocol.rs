use global_variable::{current_library_dir, get_current_fs_storage, get_current_s3_storage};
use rand::RngCore;
use std::future::Future;
use std::path::PathBuf;
use std::sync::Arc;
use storage::prelude::*;
use tauri::http::HttpRange;
use tauri::http::{header::*, status::StatusCode, MimeType, Request, Response, ResponseBuilder};
use tokio::io::AsyncWriteExt;
use url::Position;
use url::Url;

use crate::storage::state::StorageState;
use api_server::exports::storage::DataLocationType;

fn get_shard_hex(hash: &str) -> &str {
    &hash[0..3]
}

enum ResourceKind {
    // file with hash
    File(String),
    // artifacts with hash and rest path
    Artifacts(String, String),
}

/// 支持以下两种资源的访问
/// - /asset_object/[hash]/artifacts/[rest_parts...]
/// - /asset_object/[hash]/file
/// 注意：这两个 uri 都不是本地的路径，只是协议的 url schema，实际的文件路径是在这个 handler 里面计算得到的
pub fn storage_protocol_handler(
    state: Arc<tokio::sync::Mutex<StorageState>>,
    request: &Request,
) -> Result<Response, Box<dyn std::error::Error>> {
    let request_path = {
        let parsed_path = Url::parse(request.uri())?;
        let filtered_path = &parsed_path[..Position::AfterPath];
        let path = filtered_path
            .strip_prefix("storage://localhost/")
            // the `strip_prefix` only returns None when a request is made to `https://tauri.$P` on Windows
            // where `$P` is not `localhost/*`
            .unwrap_or("");
        let path = percent_encoding::percent_decode(path.as_bytes()).decode_utf8_lossy();
        path.to_string()
    };

    let mut resp = ResponseBuilder::new();

    let request_resource_kind = {
        // path should be like /asset_object/[hash]/artifacts/[artifacts_path] or /asset_object/[hash]/file
        let path_regex = match regex::Regex::new(r"^asset_object/([^/]+)/(artifacts/.*|file)$") {
            Ok(path_regex) => path_regex,
            Err(e) => {
                tracing::error!("Failed to compile regex: {}", e);
                return resp.status(StatusCode::INTERNAL_SERVER_ERROR).body(vec![]);
            }
        };
        if let Some(captures) = path_regex.captures(request_path.as_str()) {
            let asset_object_hash = match captures.get(1) {
                Some(hash) => hash.as_str().to_string(),
                None => return resp.status(StatusCode::NOT_FOUND).body(vec![]),
            };
            let srorage_type = match captures.get(2) {
                Some(x) => {
                    if x.as_str() == "file" {
                        ResourceKind::File(asset_object_hash)
                    } else {
                        match x.as_str().strip_prefix("artifacts/") {
                            Some(rest_parts) => {
                                ResourceKind::Artifacts(asset_object_hash, rest_parts.to_string())
                            }
                            None => return resp.status(StatusCode::NOT_FOUND).body(vec![]),
                        }
                    }
                }
                None => return resp.status(StatusCode::NOT_FOUND).body(vec![]),
            };
            srorage_type
        } else {
            return resp.status(StatusCode::NOT_FOUND).body(vec![]);
        }
    };

    let library_root_dir = PathBuf::from(current_library_dir!());
    // relative_dir 是 file 或者 artifacts 目录，不是最终文件的路径
    let relative_dir = match &request_resource_kind {
        ResourceKind::File(hash) => {
            let shard_hex = get_shard_hex(hash);
            PathBuf::from(format!("files/{}/{}", shard_hex, hash))
        }
        ResourceKind::Artifacts(hash, _rest_parts) => {
            let shard_hex = get_shard_hex(hash);
            PathBuf::from(format!("artifacts/{}/{}", shard_hex, hash))
        }
    };

    // check if the file exists in the local
    //  - if exists, use fs_storage
    //  - if not, check if the file exists in the s3 storage
    let mut location = DataLocationType::Fs;
    if !library_root_dir.join(&relative_dir).exists() {
        let hash = match &request_resource_kind {
            ResourceKind::File(hash) => hash,
            ResourceKind::Artifacts(hash, _) => hash,
        };
        let state_clone = state.clone();
        let hash_clone = hash.clone();
        location = match safe_block_on(async move {
            let mut state = state_clone.lock().await;
            state.get_location(hash_clone.as_str()).await
        }) {
            Ok(location) => location,
            Err(e) => {
                tracing::error!("`state.get_location` got error: {:?}", e);
                return resp.status(StatusCode::INTERNAL_SERVER_ERROR).body(vec![]);
            }
        }
    }

    let storage: Box<dyn Storage> = match location {
        DataLocationType::Fs => {
            let storage = get_current_fs_storage!()?;
            // let storage = get_or_insert_fs_storage!(library_root_dir.to_string_lossy().to_string())?; // 和 get_current_fs_storage 等价
            Box::new(storage)
        }
        DataLocationType::S3 => {
            match safe_block_on(async move {
                let mut state = state.lock().await;
                state.get_s3_config()
            }) {
                Ok(settings) => Box::new(get_current_s3_storage!(settings)?),
                Err(e) => {
                    tracing::error!("Failed to get library settings: {:?}", e);
                    return resp.status(StatusCode::INTERNAL_SERVER_ERROR).body(vec![]);
                }
            }
        }
    };

    let relative_file_path = match &request_resource_kind {
        ResourceKind::File(_) => {
            let verbose_file_name = match storage
                .read_to_string(relative_dir.join("file.json"))
                .ok()
                .and_then(|content| serde_json::from_str::<serde_json::Value>(&content).ok())
                .and_then(|json| json["verbose_file_name"].as_str().map(|s| s.to_owned()))
            {
                Some(x) => x,
                None => {
                    tracing::error!("Failed to get verbose_file_name");
                    return resp.status(StatusCode::INTERNAL_SERVER_ERROR).body(vec![]);
                }
            };
            relative_dir.join(verbose_file_name)
        }
        ResourceKind::Artifacts(_, rest_parts) => relative_dir.join(rest_parts),
    };

    let (len, mime_type, read_bytes) = safe_block_on({
        let storage = storage.clone_box();
        let request_path = request_path.clone();
        let relative_file_path = relative_file_path.clone();
        async move {
            let len = storage.len(relative_file_path.clone()).await?;
            // Avoid requesting more bytes than file size for MIME type detection
            let read_length = std::cmp::min(len, 8192);
            let buffer = match storage
                .read_with_range(relative_file_path, 0..read_length)
                .await
            {
                Ok(buffer) => buffer,
                Err(e) => {
                    tracing::error!("Failed to read file: {:?}", e);
                    return Err(anyhow::anyhow!("Failed to read file"));
                }
            };
            let range_vec = buffer.to_vec();

            let (mime_type, read_bytes) = {
                (
                    MimeType::parse(&range_vec, &request_path),
                    // return the `magic_bytes` if we read the whole file
                    // to avoid reading it again later if this is not a range request
                    if len < 8192 { Some(range_vec) } else { None },
                )
            };

            Ok::<(u64, String, Option<Vec<u8>>), anyhow::Error>((len, mime_type, read_bytes))
        }
    })?;

    resp = resp
        .header(CONTENT_TYPE, &mime_type)
        .header(ACCESS_CONTROL_ALLOW_ORIGIN, "*");

    // handle 206 (partial range) http requests
    let response = if let Some(range_header) = request
        .headers()
        .get("range")
        .and_then(|r| r.to_str().map(|r| r.to_string()).ok())
    {
        resp = resp.header(ACCEPT_RANGES, "bytes");

        let not_satisfiable = || {
            ResponseBuilder::new()
                .status(StatusCode::RANGE_NOT_SATISFIABLE)
                .header(CONTENT_RANGE, format!("bytes */{len}"))
                .body(vec![])
        };

        // parse range header
        let ranges = if let Ok(ranges) = HttpRange::parse(&range_header, len) {
            ranges
                .iter()
                // map the output to spec range <start-end>, example: 0-499
                .map(|r| (r.start, r.start + r.length - 1))
                .collect::<Vec<_>>()
        } else {
            return not_satisfiable();
        };

        /// The Maximum bytes we send in one range
        const MAX_LEN: u64 = 1000 * 1024;

        // single-part range header
        if ranges.len() == 1 {
            let &(start, mut end) = ranges.first().unwrap();

            // check if a range is not satisfiable
            //
            // this should be already taken care of by the range parsing library
            // but checking here again for extra assurance
            if start >= len || end >= len || end < start {
                return not_satisfiable();
            }

            // adjust end byte for MAX_LEN
            end = start + (end - start).min(len - start).min(MAX_LEN - 1);

            // calculate number of bytes needed to be read
            let nbytes = end + 1;

            let buf = safe_block_on(async move {
                let buf = storage
                    .read_with_range(relative_file_path, start..nbytes)
                    .await?
                    .to_vec();
                Ok::<Vec<u8>, anyhow::Error>(buf)
            })?;

            resp = resp.header(CONTENT_RANGE, format!("bytes {start}-{end}/{len}"));
            resp = resp.header(CONTENT_LENGTH, end + 1 - start);
            resp = resp.status(StatusCode::PARTIAL_CONTENT);
            resp.body(buf)
        } else {
            let ranges = ranges
                .iter()
                .filter_map(|&(start, mut end)| {
                    // filter out unsatisfiable ranges
                    //
                    // this should be already taken care of by the range parsing library
                    // but checking here again for extra assurance
                    if start >= len || end >= len || end < start {
                        None
                    } else {
                        // adjust end byte for MAX_LEN
                        end = start + (end - start).min(len - start).min(MAX_LEN - 1);
                        Some((start, end))
                    }
                })
                .collect::<Vec<_>>();

            let boundary = random_boundary();
            let boundary_sep = format!("\r\n--{boundary}\r\n");
            let boundary_closer = format!("\r\n--{boundary}\r\n");

            resp = resp.header(
                CONTENT_TYPE,
                format!("multipart/byteranges; boundary={boundary}"),
            );

            let buf = safe_block_on(async move {
                // multi-part range header
                let mut buf = Vec::new();

                for (end, start) in ranges {
                    // a new range is being written, write the range boundary
                    buf.write_all(boundary_sep.as_bytes()).await?;

                    // write the needed headers `Content-Type` and `Content-Range`
                    buf.write_all(format!("{CONTENT_TYPE}: {mime_type}\r\n").as_bytes())
                        .await?;
                    buf.write_all(
                        format!("{CONTENT_RANGE}: bytes {start}-{end}/{len}\r\n").as_bytes(),
                    )
                    .await?;

                    // write the separator to indicate the start of the range body
                    buf.write_all("\r\n".as_bytes()).await?;

                    // calculate number of bytes needed to be read
                    let nbytes = end + 1 - start;

                    let local_buf = storage
                        .read_with_range(relative_file_path.clone(), start..nbytes)
                        .await?
                        .to_vec();

                    buf.extend_from_slice(&local_buf);
                }
                // all ranges have been written, write the closing boundary
                buf.write_all(boundary_closer.as_bytes()).await?;

                Ok::<Vec<u8>, anyhow::Error>(buf)
            })?;
            resp.body(buf)
        }
    } else {
        // avoid reading the file if we already read it
        // as part of mime type detection
        let buf = if let Some(b) = read_bytes {
            b
        } else {
            safe_block_on(async move {
                let local_buf = storage.read(relative_file_path).await?.to_vec();
                Ok::<Vec<u8>, anyhow::Error>(local_buf)
            })?
        };
        resp = resp.header(CONTENT_LENGTH, len);
        resp.body(buf)
    };

    response
}

fn random_boundary() -> String {
    let mut x = [0_u8; 30];
    rand::thread_rng().fill_bytes(&mut x);
    (x[..])
        .iter()
        .map(|&x| format!("{x:x}"))
        .fold(String::new(), |mut a, x| {
            a.push_str(x.as_str());
            a
        })
}

pub(crate) fn safe_block_on<F>(task: F) -> F::Output
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    let handle = tokio::runtime::Handle::try_current().unwrap();
    let (tx, rx) = std::sync::mpsc::sync_channel(1);
    let handle_ = handle.clone();
    handle.spawn_blocking(move || {
        tx.send(handle_.block_on(task)).unwrap();
    });
    rx.recv().unwrap()
}
