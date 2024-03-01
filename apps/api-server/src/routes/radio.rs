use crate::{Ctx, R};
use rspc::Router;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::{info, warn};
pub fn get_routes() -> Router<Ctx> {
    let router = R.router().procedure(
        "find_one",
        R.query(|ctx, hash: String| async move {
            let artifacts_dir = ctx.library.artifacts_dir.clone();
            let reader = RadioReader::new(artifacts_dir, hash);
            serde_json::to_value::<Vec<RadioData>>(reader.read().unwrap_or_default())
                .unwrap_or_default()
        }),
    );
    router
}

#[derive(Debug, Deserialize, Serialize)]
struct RadioData {
    start_timestamp: u32,
    end_timestamp: u32,
    text: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct RadioReader {
    /// path 是 transcript.txt 的完整路径
    path: PathBuf,
}

impl RadioReader {
    pub fn new(dir: PathBuf, file_hash: String) -> Self {
        let path = RadioReader::get_file_path(dir, file_hash).join("transcript.txt");
        Self { path }
    }

    fn get_file_path(dir: PathBuf, file_hash: String) -> PathBuf {
        dir.join(file_hash)
    }

    /// 读取 transcript.txt 文件内容
    /// 文件格式为 JSON: [{"start_timestamp":0,"end_timestamp":1880,"text":"..."}]
    /// 返回 RadioData
    pub fn read(&self) -> anyhow::Result<Vec<RadioData>> {
        info!("path {}", &self.path.display());
        let content = std::fs::read_to_string(&self.path)?;
        Ok(serde_json::from_str(&content)?)
    }
}
