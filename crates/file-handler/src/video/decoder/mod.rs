#[cfg(feature = "ffmpeg-binary")]
mod ffmpeg;
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
use std::path::Path;
use tracing::debug;

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

#[cfg(feature = "ffmpeg-binary")]
pub struct VideoDecoder {
    video_file_path: std::path::PathBuf,
    binary_file_path: std::path::PathBuf,
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

        Ok(Self {
            video_file_path: filename.as_ref().to_path_buf(),
            binary_file_path,
        })
    }
}

#[cfg(feature = "ffmpeg-binary")]
impl VideoDecoder {
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
