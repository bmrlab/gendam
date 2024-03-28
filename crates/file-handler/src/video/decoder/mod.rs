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
use std::{fs, path::Path};

use super::FRAME_FILE_EXTENSION;

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
    index: usize,
    codec_type: String, // video, audio
    width: Option<usize>,
    height: Option<usize>,
    avg_frame_rate: String,
    duration: String,
    bit_rate: String,
    nb_frames: String,
    time_base: String,
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
pub struct AudioMetadata {
    pub bit_rate: usize,
    pub duration: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoMetadata {
    pub width: usize,
    pub height: usize,
    /// video duration in seconds
    pub duration: f64,
    pub bit_rate: usize,
    pub avg_frame_rate: VideoAvgFrameRate,
    pub audio: Option<AudioMetadata>,
}

impl VideoMetadata {
    pub fn with_audio(&mut self, metadata: AudioMetadata) {
        self.audio = Some(metadata);
    }
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

impl From<&RawProbeStreamOutput> for VideoMetadata {
    fn from(stream: &RawProbeStreamOutput) -> Self {
        Self {
            width: stream.width.unwrap_or(0),
            height: stream.height.unwrap_or(0),
            duration: stream.duration.parse().unwrap_or(0.0),
            bit_rate: stream.bit_rate.parse().unwrap_or(0),
            avg_frame_rate: VideoAvgFrameRate::from(stream.avg_frame_rate.clone()),
            audio: None,
        }
    }
}

impl From<&RawProbeStreamOutput> for AudioMetadata {
    fn from(stream: &RawProbeStreamOutput) -> Self {
        Self {
            bit_rate: stream.bit_rate.parse().unwrap_or(0),
            duration: stream.duration.parse().unwrap_or(0.0),
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
    pub async fn new(filename: impl AsRef<Path>) -> anyhow::Result<Self> {
        let current_exe_path = std::env::current_exe().expect("failed to get current executable");
        let current_dir = current_exe_path
            .parent()
            .expect("failed to get parent directory");
        let binary_file_path = current_dir.join("ffmpeg");
        let ffprobe_file_path = current_dir.join("ffprobe");

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
                "-show_streams",
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

                    match raw_output
                        .streams
                        .iter()
                        .find(|stream| stream.codec_type == "video")
                    {
                        Some(stream) => {
                            let mut metadata = VideoMetadata::from(stream);

                            if let Some(audio_stream) = raw_output
                                .streams
                                .iter()
                                .find(|stream| stream.codec_type == "audio")
                            {
                                metadata.with_audio(AudioMetadata::from(audio_stream));
                            }

                            Ok(metadata)
                        }
                        None => {
                            bail!("Failed to find video stream");
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
        fs::create_dir_all(frames_dir.as_ref())?;
        match std::process::Command::new(&self.binary_file_path)
            .args([
                "-i",
                self.video_file_path
                    .to_str()
                    .expect("invalid video file path"),
                "-vf",
                "scale='if(gte(iw,ih)*sar,768,-1)':'if(gte(iw,ih)*sar, -1, 768)', fps=1",
                "-vsync",
                "vfr",
                "-compression_level",
                "9",
                frames_dir
                    .as_ref()
                    .join(format!("%d000.{}", FRAME_FILE_EXTENSION))
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
        let video_decoder = VideoDecoder::new(
            "/Users/zhuo/Desktop/file_v2_f566a493-ad1b-4324-b16f-0a4c6a65666g 2.MP4",
        );

        let frames_fut = video_decoder.save_video_frames("/Users/zhuo/Desktop/frames");
        let audio_fut = video_decoder.save_video_audio("/Users/zhuo/Desktop/audio.wav");

        let (_res1, _res2) = tokio::join!(frames_fut, audio_fut);
    }

    #[cfg(feature = "ffmpeg-binary")]
    {
        let video_decoder = VideoDecoder::new("/Users/zhuo/Desktop/1-4 插件-整页截屏.mp4")
            .await
            .expect("failed to find ffmpeg binary file");

        // let frames_fut = video_decoder.save_video_frames("/Users/zhuo/Desktop/frames");
        // let audio_fut = video_decoder.save_video_audio("/Users/zhuo/Desktop/audio.wav");

        // let (_res1, _res2) = tokio::join!(frames_fut, audio_fut);

        // let _ = video_decoder
        //     .save_video_frames("/Users/zhuo/Desktop/frames")
        //     .await;

        let metadata = video_decoder
            .get_video_metadata()
            .await
            .expect("failed to get video metadata");
        println!("{metadata:#?}");
    }
}
