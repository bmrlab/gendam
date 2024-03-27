use reqwest;
use tokio::{
    fs::{self, File},
    io::AsyncWriteExt,
};
use tracing::info;

pub struct FileDownloadConfig {
    pub url: String,
    pub resources_dir: std::path::PathBuf,
}

pub struct FileDownload {
    url: String,
    resources_dir: std::path::PathBuf,
}

impl Default for FileDownloadConfig {
    fn default() -> Self {
        Self {
            url: "https://tezign-ai-models.oss-cn-beijing.aliyuncs.com".to_string(),
            resources_dir: std::path::PathBuf::from("resources"),
        }
    }
}

impl FileDownload {
    pub fn new(config: FileDownloadConfig) -> Self {
        Self {
            url: config.url,
            resources_dir: config.resources_dir,
        }
    }

    pub async fn download_to_path_if_not_exists(
        &self,
        uri: impl AsRef<std::path::Path>,
        file_path: impl AsRef<std::path::Path>,
    ) -> anyhow::Result<std::path::PathBuf> {
        let file_path = file_path.as_ref().to_path_buf();
        info!("check file path: {:?}", file_path);
        if file_path.exists() {
            return Ok(file_path);
        }

        let temp_download_path = file_path.with_extension("temp");
        let download_url = format!("{}/{}", self.url, uri.as_ref().to_str().unwrap());

        let mut response = reqwest::get(&download_url).await?;

        // create parent folder
        if let Some(parent_dir) = file_path.parent() {
            fs::create_dir_all(parent_dir).await?;
        }

        let mut file = File::create(&temp_download_path).await?;
        while let Some(chunk) = response.chunk().await? {
            file.write_all(&chunk).await?;
        }
        fs::rename(&temp_download_path, &file_path).await?;

        info!("file {:?} downloaded", file_path);

        Ok(file_path)
    }

    pub async fn download_if_not_exists(
        &self,
        uri: impl AsRef<std::path::Path>,
    ) -> anyhow::Result<std::path::PathBuf> {
        let file_path = self.resources_dir.join(&uri);
        self.download_to_path_if_not_exists(uri, file_path).await
    }
}
