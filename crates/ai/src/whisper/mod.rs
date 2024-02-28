use anyhow::{anyhow, bail};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub struct Whisper {
    binary_path: PathBuf,
    model_path: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct WhisperTranscriptionOffset {
    from: i64,
    to: i64,
}

#[derive(Debug, Serialize, Deserialize)]
struct WhisperTranscription {
    pub offsets: WhisperTranscriptionOffset,
    pub text: String,
    // TODO enable it when we need character level timestamp
    // pub tokens: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WhisperResult {
    transcription: Vec<WhisperTranscription>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WhisperItem {
    pub start_timestamp: i64,
    pub end_timestamp: i64,
    pub text: String,
}

impl WhisperResult {
    pub fn items(&self) -> Vec<WhisperItem> {
        self.transcription
            .iter()
            .map(|item| WhisperItem {
                start_timestamp: item.offsets.from,
                end_timestamp: item.offsets.to,
                text: item.text.clone(),
            })
            .collect()
    }
}

pub struct WhisperParams {}

impl Default for WhisperParams {
    fn default() -> Self {
        Self {}
    }
}

impl Whisper {
    pub async fn new(resources_dir: impl AsRef<Path>) -> anyhow::Result<Self> {
        let download = file_downloader::FileDownload::new(file_downloader::FileDownloadConfig {
            resources_dir: resources_dir.as_ref().to_path_buf(),
            ..Default::default()
        });

        let model_path = download
            .download_if_not_exists("whisper-ggml-base.bin")
            .await?
            .to_str()
            .ok_or(anyhow!("invalid path"))?
            .to_string();

        let binary_path = download.download_if_not_exists("whisper/main").await?;
        download
            .download_if_not_exists("whisper/ggml-metal.metal")
            .await?;

        Ok(Self {
            binary_path,
            model_path,
        })
    }

    pub fn transcribe(
        &mut self,
        audio_file_path: impl AsRef<Path>,
        _params: Option<WhisperParams>,
    ) -> anyhow::Result<WhisperResult> {
        let output_file_path = audio_file_path.as_ref().with_file_name("transcript");

        match std::process::Command::new(self.binary_path.clone())
            .args([
                "-l",
                "auto",
                "-f",
                audio_file_path.as_ref().to_str().unwrap(),
                "-m",
                &self.model_path,
                "-ojf",
                "-of",
                output_file_path.to_str().unwrap(),
                "-tr",
            ])
            .output()
        {
            Ok(output) => {
                if !output.status.success() {
                    bail!("{}", String::from_utf8_lossy(&output.stderr));
                }
            }
            Err(e) => {
                bail!("failed to run subprocess {}", e);
            }
        }

        let transcript = std::fs::read_to_string(output_file_path.with_extension("json"))?;
        let items: WhisperResult = serde_json::from_str(&transcript)?;

        Ok(items)
    }
}

#[test_log::test(tokio::test)]
async fn test_whisper() {
    use tracing::{debug, error};

    let mut whisper =
        Whisper::new("/Users/zhuo/Library/Application Support/cc.musedam.local/resources")
            .await
            .unwrap();
    match whisper
        .transcribe("/Users/zhuo/Library/Application Support/cc.musedam.local/libraries/98f19afbd2dee7fa6415d5f523d36e8322521e73fd7ac21332756330e836c797/artifacts/1aaa451c0bee906e2d1f9cac21ebb2ef5f2f82b2f87ec928fc04b58cbceda60b/audio.wav", None)
    {
        Ok(result) => {
            for item in result.items() {
                debug!(
                    "[{}] [{}] {}",
                    item.start_timestamp, item.end_timestamp, item.text
                );
            }
        }
        Err(e) => {
            error!("failed to transcribe: {}", e);
        }
    }
}
