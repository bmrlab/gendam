use crate::{
    ctx::traits::CtxWithLibrary,
    download::{
        DownloadReporter, {file_name_from_url, SimpleReporter},
    },
};
use downloader::{Download, Downloader};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use specta::Type;
use std::path::{Path, PathBuf};
use strum_macros::{AsRefStr, EnumString};
use tracing::error;

#[derive(Serialize, Deserialize, Type, Debug, Clone)]
pub struct ModelArtifact {
    url: String,
    checksum: String,
}

#[derive(Serialize, Deserialize, AsRefStr, EnumString, PartialEq, Eq, Hash, Clone, Type, Debug)]
pub enum AIModelCategory {
    // ImageEmbedding,
    MultiModalEmbedding,
    ImageCaption,
    AudioTranscript,
    TextEmbedding,
    LLM,
}

#[derive(AsRefStr, Debug, Clone, Serialize, Deserialize, PartialEq, Eq, EnumString, Type)]
pub enum ConcreteModelType {
    BLIP,
    CLIP,
    Moondream,
    OrtTextEmbedding,
    Whisper,
    Yolo,
    Qwen2,
    OpenAI,
    AzureOpenAI,
    LLaVAPhi3Mini,
}

// TODO: rename, in order to distinguish from ai::AIModel
#[derive(Serialize, Deserialize, Type, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub struct AIModel {
    pub id: String,
    pub title: String,
    pub description: String,
    pub categories: Vec<AIModelCategory>,
    pub artifacts_dir: PathBuf,
    pub artifacts: Vec<ModelArtifact>,
    /// use this to find corresponding concrete struct
    pub model_type: ConcreteModelType,
    /// use these params to instantiate the model
    pub params: Value,
    pub dim: Option<u32>,
}

#[derive(Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ModelDownloadStatus {
    total_bytes: String,
    downloaded_bytes: String,
}

#[derive(Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AIModelStatus {
    pub downloaded: bool,
    pub download_status: Option<ModelDownloadStatus>,
}

#[derive(Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AIModelResult {
    pub info: AIModel,
    pub status: AIModelStatus,
}

pub fn load_model_list(resources_dir: impl AsRef<Path>) -> anyhow::Result<Vec<AIModel>> {
    let resources_dir = resources_dir.as_ref();
    let model_list_file = resources_dir.join("model_list.json");

    // read json from model_list_file
    let model_list = std::fs::read_to_string(model_list_file)
        .map_err(|e| anyhow::anyhow!("Failed to read model list: {}", e))?;

    let model_list: Vec<AIModel> = serde_json::from_str(&model_list)
        .map_err(|e| anyhow::anyhow!("Invalid model list format: {}", e))?;

    Ok(model_list)
}

pub fn get_model_info_by_id(ctx: &dyn CtxWithLibrary, model_id: &str) -> anyhow::Result<AIModel> {
    let model_list = load_model_list(ctx.get_resources_dir())?;

    let model = model_list
        .iter()
        .find(|v| v.id == model_id)
        .ok_or(anyhow::anyhow!("model not found: {}", model_id))?;

    Ok(model.to_owned())
}

pub fn get_model_status(ctx: &dyn CtxWithLibrary, model: &AIModel) -> AIModelStatus {
    if let Ok(download_status) = ctx.download_status() {
        let mut total_bytes: u64 = 0;
        let mut downloaded_bytes: u64 = 0;
        let mut is_downloading = false;

        model.artifacts.iter().for_each(|v| {
            if let Some(status) = download_status
                .iter()
                .find(|t| t.file_name == file_name_from_url(&v.url))
            {
                if status.exit_code.is_none() {
                    is_downloading = true;
                }
                let current_total_bytes = status.total_bytes.unwrap_or(status.downloaded_bytes);
                total_bytes += current_total_bytes;

                match &status.exit_code {
                    Some(0) => {
                        downloaded_bytes += current_total_bytes;
                    }
                    _ => {
                        downloaded_bytes += status.downloaded_bytes;
                    }
                }
            }
        });

        if is_downloading {
            return AIModelStatus {
                downloaded: false,
                download_status: Some(ModelDownloadStatus {
                    total_bytes: total_bytes.to_string(),
                    downloaded_bytes: downloaded_bytes.to_string(),
                }),
            };
        }
    }

    // 未在下载
    let downloaded = model.artifacts.iter().all(|v| {
        let path = file_name_from_url(&v.url);
        let artifact_path = ctx
            .get_resources_dir()
            .join(&model.artifacts_dir)
            .join(path);
        artifact_path.exists() // FIXME 文件存在可能是下载了一半但是失败 && v.checksum == compute_checksum(&artifact_path)
    });

    AIModelStatus {
        downloaded,
        download_status: None,
    }
}

pub fn trigger_model_download(
    resources_dir: impl AsRef<Path>,
    model: &AIModel,
    reporter: DownloadReporter,
) -> anyhow::Result<()> {
    let target_dir = resources_dir.as_ref().join(&model.artifacts_dir);
    std::fs::create_dir_all(&target_dir)
        .map_err(|e| anyhow::anyhow!("Failed to create directory: {}", e))?;

    let mut downloader = Downloader::builder()
        .download_folder(&target_dir)
        .parallel_requests(4)
        .build()
        .map_err(|e| anyhow::anyhow!("Failed to create downloader: {}", e))?;

    let downloads = model
        .artifacts
        .iter()
        .map(|v| {
            let dl = Download::new(&v.url);
            dl.progress(SimpleReporter::create(
                file_name_from_url(&v.url),
                target_dir.clone(),
                v.url.clone(),
                reporter.clone(),
            ))
        })
        .collect::<Vec<_>>();

    std::thread::spawn(move || {
        let results = downloader.download(&downloads);

        match results {
            Ok(results) => {
                for result in results {
                    if let Err(e) = result {
                        error!("failed to download model: {}", e);
                    }
                }
            }
            Err(e) => {
                error!("failed to download model: {}", e);
            }
        }
    });

    Ok(())
}
