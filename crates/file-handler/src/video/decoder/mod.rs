#[cfg(feature = "ffmpeg-dylib")]
mod ffmpeg_lib;
#[cfg(feature = "ffmpeg-dylib")]
mod transcode;
#[cfg(feature = "ffmpeg-dylib")]
mod utils;

#[cfg(feature = "ffmpeg-dylib")]
pub struct VideoDecoder {
    video_file_path: std::path::PathBuf,
}

use anyhow::bail;
use serde::{Deserialize, Serialize};
use std::{fs, os::unix::fs::PermissionsExt, path::Path};

#[cfg(feature = "ffmpeg-dylib")]
impl VideoDecoder {
    pub fn new(filename: impl AsRef<Path>) -> Self {
        debug!("Successfully opened {}", filename.as_ref().display());

        let decoder = Self {
            video_file_path: filename.as_ref().to_path_buf(),
        };

        decoder
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RawProbeStreamOutput {
    width: usize,
    height: usize,
    avg_frame_rate: String,
    duration: String,
    bit_rate: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RawProbeOutput {
    streams: Vec<RawProbeStreamOutput>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoAvgFrameRate {
    pub numerator: usize,
    pub denominator: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoMetadata {
    pub width: usize,
    pub height: usize,
    /// video duration in seconds
    pub duration: f64,
    pub bit_rate: usize,
    pub avg_frame_rate: VideoAvgFrameRate,
}

impl From<String> for VideoAvgFrameRate {
    fn from(s: String) -> Self {
        let (numerator, denominator) = s
            .split_once('/')
            .map(|(numerator, denominator)| (numerator, denominator))
            .unwrap_or((s.as_str(), "1"));
        Self {
            numerator: numerator.parse().unwrap_or(0),
            denominator: denominator.parse().unwrap_or(1),
        }
    }
}

impl From<RawProbeStreamOutput> for VideoMetadata {
    fn from(stream: RawProbeStreamOutput) -> Self {
        Self {
            width: stream.width,
            height: stream.height,
            duration: stream.duration.parse().unwrap_or(0.0),
            bit_rate: stream.bit_rate.parse().unwrap_or(0),
            avg_frame_rate: VideoAvgFrameRate::from(stream.avg_frame_rate),
        }
    }
}

#[cfg(feature = "ffmpeg-binary")]
pub struct VideoDecoder {
    video_file_path: std::path::PathBuf,
    binary_file_path: std::path::PathBuf,
    ffprobe_file_path: std::path::PathBuf,
}

#[cfg(feature = "ffmpeg-binary")]
impl VideoDecoder {
    pub async fn new(
        filename: impl AsRef<Path>,
        resources_dir: impl AsRef<Path>,
    ) -> anyhow::Result<Self> {
        let download = file_downloader::FileDownload::new(file_downloader::FileDownloadConfig {
            resources_dir: resources_dir.as_ref().to_path_buf(),
            ..Default::default()
        });

        let binary_file_path = download.download_if_not_exists("ffmpeg").await?;
        let ffprobe_file_path = download.download_if_not_exists("ffprobe").await?;

        // set binary permission to executable
        let mut perms = fs::metadata(&binary_file_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&binary_file_path, perms)?;
        let mut perms = fs::metadata(&ffprobe_file_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&ffprobe_file_path, perms)?;

        Ok(Self {
            video_file_path: filename.as_ref().to_path_buf(),
            binary_file_path,
            ffprobe_file_path,
        })
    }
}

#[cfg(feature = "ffmpeg-binary")]
impl VideoDecoder {
    pub async fn get_video_metadata(&self) -> anyhow::Result<VideoMetadata> {
        match std::process::Command::new(&self.ffprobe_file_path)
            .args([
                "-v",
                "error",
                "-select_streams",
                "v:0",
                "-show_entries",
                "stream=width,height,duration,bit_rate,avg_frame_rate",
                "-of",
                "json",
                self.video_file_path
                    .to_str()
                    .expect("invalid video file path"),
            ])
            .output()
        {
            Ok(output) => match String::from_utf8(output.stdout) {
                Ok(result) => {
                    let raw_output: RawProbeOutput = serde_json::from_str(&result)?;

                    match raw_output.streams.into_iter().next().take() {
                        Some(stream) => Ok(VideoMetadata::from(stream)),
                        None => {
                            bail!("Failed to get video stream")
                        }
                    }
                }
                Err(e) => {
                    bail!("Failed to get video metadata: {e}");
                }
            },
            Err(e) => {
                bail!("Failed to get video metadata: {e}");
            }
        }
    }

    pub async fn save_video_frames(&self, frames_dir: impl AsRef<Path>) -> anyhow::Result<()> {
        match std::process::Command::new(&self.binary_file_path)
            .args([
                "-i",
                self.video_file_path
                    .to_str()
                    .expect("invalid video file path"),
                "-vf",
                &format!("fps=1"),
                "-vsync",
                "vfr",
                frames_dir
                    .as_ref()
                    .join("%d000.png")
                    .to_str()
                    .expect("invalid frames dir path"),
            ])
            .output()
        {
            Ok(output) => {
                if !output.status.success() {
                    bail!(
                        "Failed to save video frames: {}",
                        String::from_utf8_lossy(&output.stderr)
                    );
                }
            }
            Err(e) => {
                bail!("Failed to save video frames: {e}");
            }
        }

        Ok(())
    }

    pub async fn save_video_audio(&self, audio_path: impl AsRef<Path>) -> anyhow::Result<()> {
        match std::process::Command::new(&self.binary_file_path)
            .args([
                "-i",
                self.video_file_path
                    .to_str()
                    .expect("invalid video file path"),
                "-vn",
                "-ar",
                // the rate must be 16KHz to fit whisper.cpp
                "16000",
                "-ac",
                "1",
                audio_path.as_ref().to_str().expect("invalid audio path"),
            ])
            .output()
        {
            Ok(output) => {
                if !output.status.success() {
                    bail!(
                        "Failed to save video frames: {}",
                        String::from_utf8_lossy(&output.stderr)
                    );
                }
            }
            Err(e) => {
                bail!("Failed to save video frames: {e}");
            }
        }

        Ok(())
    }
}

#[cfg(feature = "ffmpeg-dylib")]
impl VideoDecoder {
    pub async fn save_video_frames(&self, frames_dir: impl AsRef<Path>) -> anyhow::Result<()> {
        ffmpeg_lib::save_video_frames(self.video_file_path.to_path_buf(), frames_dir)
    }

    pub async fn save_video_audio(&self, audio_path: impl AsRef<Path>) -> anyhow::Result<()> {
        ffmpeg_lib::save_video_audio(self.video_file_path.to_path_buf(), audio_path)
    }
}

#[test_log::test(tokio::test)]
async fn test_video_decoder() {
    #[cfg(feature = "ffmpeg-dylib")]
    {
        let video_decoder = VideoDecoder::new("/Users/zhuo/Desktop/20240218-143801.mp4");

        let frames_fut = video_decoder.save_video_frames("/Users/zhuo/Desktop/frames");
        let audio_fut = video_decoder.save_video_audio("/Users/zhuo/Desktop/audio.wav");

        let (_res1, _res2) = tokio::join!(frames_fut, audio_fut);
    }

    #[cfg(feature = "ffmpeg-binary")]
    {
        let video_decoder = VideoDecoder::new("/Users/zhuo/Desktop/20240218-143801.mp4", "/Users/zhuo/dev/tezign/bmrlab/tauri-dam-test-playground/apps/desktop/src-tauri/resources/ffmpeg").await.expect("failed to find ffmpeg binary file");

        let frames_fut = video_decoder.save_video_frames("/Users/zhuo/Desktop/frames");
        let audio_fut = video_decoder.save_video_audio("/Users/zhuo/Desktop/audio.wav");

        let (_res1, _res2) = tokio::join!(frames_fut, audio_fut);
    }
}
