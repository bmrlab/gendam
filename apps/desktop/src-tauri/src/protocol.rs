use rand::RngCore;
use std::future::Future;
use storage::Storage;
use tokio::io::AsyncWriteExt;
use url::Position;
use url::Url;

use global_variable::write_storage_map;
use tauri::http::HttpRange;
use tauri::http::{header::*, status::StatusCode, MimeType, Request, Response, ResponseBuilder};

fn get_or_insert_storage(root_path: String) -> Storage {
    let mut map = write_storage_map!().unwrap();
    map.entry(root_path.clone())
        .or_insert_with(|| Storage::new_fs(&root_path).unwrap())
        .clone()
}

pub fn asset_protocol_handler(request: &Request) -> Result<Response, Box<dyn std::error::Error>> {
    let parsed_path = Url::parse(request.uri())?;
    let filtered_path = &parsed_path[..Position::AfterPath];
    let path = filtered_path
        .strip_prefix("storage://localhost/")
        // the `strip_prefix` only returns None when a request is made to `https://tauri.$P` on Windows
        // where `$P` is not `localhost/*`
        .unwrap_or("");
    let path = percent_encoding::percent_decode(path.as_bytes())
        .decode_utf8_lossy()
        .to_string();

    let mut resp = ResponseBuilder::new();

    // split path
    // get `root path` and `relative path`
    let part_split: Vec<&str> = path.split('/').collect();
    let index_res = part_split
        .iter()
        .position(|&x| x == "artifacts" || x == "files");
    if index_res.is_none() {
        return resp.status(StatusCode::NOT_FOUND).body(vec![]);
    }
    let index = index_res.unwrap();
    // extract the part starting from "artifacts or files"
    let root_path = part_split[..index].join("/");
    let relative_path = part_split[index..].join("/");
    let storage = get_or_insert_storage(root_path);

    let relative_path_clone = relative_path.clone();
    let storage_clone = storage.clone();
    let (len, mime_type, read_bytes) = safe_block_on(async move {
        let len = storage_clone.len(&relative_path_clone).await?;
        let range_vec = storage_clone
            .read_with_range(&relative_path_clone, 0..8192)
            .await?
            .to_vec();

        let (mime_type, read_bytes) = {
            (
                MimeType::parse(&range_vec, &path),
                // return the `magic_bytes` if we read the whole file
                // to avoid reading it again later if this is not a range request
                if len < 8192 { Some(range_vec) } else { None },
            )
        };

        Ok::<(u64, String, Option<Vec<u8>>), anyhow::Error>((len, mime_type, read_bytes))
    })?;

    resp = resp.header(CONTENT_TYPE, &mime_type);

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
                    .read_with_range(&relative_path, start..nbytes)
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
                        .read_with_range(&relative_path, start..nbytes)
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
                let local_buf = storage.read(&relative_path).await?.to_vec();
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
