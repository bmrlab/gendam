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

use crate::metadata::{
    audio::AudioMetadata,
    video::{VideoAvgFrameRate, VideoMetadata},
};

use super::FRAME_FILE_EXTENSION;
use anyhow::bail;
use serde::{Deserialize, Serialize};
use std::path::Path;

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

/*
 * 有一些 stream 的 codec_type 比如 "hevc"，它们的 ffprobe 返回数据中没有 duration 等字段
 * 安全点就所有字段都是 option
 */
#[derive(Debug, Clone, Serialize, Deserialize)]
struct RawProbeStreamOutput {
    index: usize,
    codec_type: String, // video, audio
    width: Option<usize>,
    height: Option<usize>,
    avg_frame_rate: Option<String>,
    duration: Option<String>,
    bit_rate: Option<String>,
    nb_frames: Option<String>,
    time_base: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RawProbeOutput {
    streams: Vec<RawProbeStreamOutput>,
}

impl From<&RawProbeStreamOutput> for VideoMetadata {
    fn from(stream: &RawProbeStreamOutput) -> Self {
        let stream = stream.clone();
        Self {
            width: stream.width.unwrap_or(0),
            height: stream.height.unwrap_or(0),
            duration: stream.duration.unwrap_or_default().parse().unwrap_or(0.0),
            bit_rate: stream.bit_rate.unwrap_or_default().parse().unwrap_or(0),
            avg_frame_rate: VideoAvgFrameRate::from(
                stream.avg_frame_rate.unwrap_or_default().clone(),
            ),
            audio: None,
        }
    }
}

impl From<&RawProbeStreamOutput> for AudioMetadata {
    fn from(stream: &RawProbeStreamOutput) -> Self {
        let stream = stream.clone();
        Self {
            bit_rate: stream.bit_rate.unwrap_or_default().parse().unwrap_or(0),
            duration: stream.duration.unwrap_or_default().parse().unwrap_or(0.0),
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
    pub fn new(filename: impl AsRef<Path>) -> anyhow::Result<Self> {
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
    pub fn get_video_metadata(&self) -> anyhow::Result<VideoMetadata> {
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

    pub async fn save_video_thumbnail(
        &self,
        thumbnail_path: impl AsRef<Path>,
        seconds: Option<u64>,
    ) -> anyhow::Result<()> {
        let seconds_string = {
            let seconds_duration = std::time::Duration::from_millis(seconds.unwrap_or(0));
            let hours = seconds_duration.as_secs() / 3600;
            let minutes = (seconds_duration.as_secs() % 3600) / 60;
            let seconds = (seconds_duration.as_secs() % 3600) % 60;

            format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
        };

        match std::process::Command::new(&self.binary_file_path)
            .args([
                "-i",
                self.video_file_path
                    .to_str()
                    .expect("invalid video file path"),
                "-ss",
                &seconds_string,
                "-vframes",
                "1",
                "-compression_level",
                "9",
                thumbnail_path.as_ref().to_string_lossy().as_ref(),
            ])
            .output()
        {
            Ok(output) => {
                if !output.status.success() {
                    bail!(
                        "Failed to save video thumbnail: {}",
                        String::from_utf8_lossy(&output.stderr)
                    );
                }

                Ok(())
            }
            Err(e) => {
                bail!("Failed to save video thumbnail: {e}");
            }
        }
    }

    pub async fn save_video_frames(&self, frames_dir: impl AsRef<Path>) -> anyhow::Result<()> {
        // 单独提取 timestamp 为 0 的帧
        match std::process::Command::new(&self.binary_file_path)
            .args([
                "-i",
                self.video_file_path
                    .to_str()
                    .expect("invalid video file path"),
                "-vf",
                "scale='if(gte(iw,ih)*sar,768,-1)':'if(gte(iw,ih)*sar, -1, 768)',select=eq(n\\,0)",
                "-vsync",
                "vfr",
                "-compression_level",
                "9",
                frames_dir
                    .as_ref()
                    .join(format!("0.{}", FRAME_FILE_EXTENSION))
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

    pub fn save_video_segment(
        &self,
        verbose_file_name: &str,
        output_dir: impl AsRef<Path>,
        milliseconds_from: u32,
        milliseconds_to: u32,
    ) -> anyhow::Result<()> {
        fn format_seconds(milliseconds: u32) -> String {
            let seconds_duration = std::time::Duration::from_millis(milliseconds as u64);
            let hours = seconds_duration.as_secs() / 3600;
            let minutes = (seconds_duration.as_secs() % 3600) / 60;
            let seconds = (seconds_duration.as_secs() % 3600) % 60;
            format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
        }
        let seconds_string_from = format_seconds(milliseconds_from);
        let seconds_string_to = format_seconds(milliseconds_to);
        // ffmpeg -i filename.mp4 -ss 00:00:02 -to 00:00:04 -c copy "[00:00:02,00:00:04] filename.mp4"
        let (file_name_wo_ext, file_ext) = match verbose_file_name.rsplit_once('.') {
            Some((wo_ext, ext)) => (wo_ext.to_owned(), format!(".{}", ext)),
            None => (verbose_file_name.to_owned(), "".to_string()),
        };
        let output_full_path = output_dir.as_ref().join(format!(
            "{} [{},{}]{}",
            file_name_wo_ext, milliseconds_from, milliseconds_to, file_ext
        ));
        match std::process::Command::new(&self.binary_file_path)
            .args([
                "-i",
                self.video_file_path
                    .to_str()
                    .expect("invalid video file path"),
                "-ss",
                &seconds_string_from,
                "-to",
                &seconds_string_to,
                // "-c",
                // "copy",  // "copy" codec 有时候会让有些帧空白, 删除这个参数, 导出文件会大一点但稳定
                output_full_path.to_string_lossy().as_ref(),
            ])
            .output()
        {
            Ok(output) => {
                if !output.status.success() {
                    bail!(
                        "Failed to save video segment: {}",
                        String::from_utf8_lossy(&output.stderr)
                    );
                }
                Ok(())
            }
            Err(e) => {
                bail!("Failed to save video segment: {e}");
            }
        }
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
            .expect("failed to find ffmpeg binary file");

        // let frames_fut = video_decoder.save_video_frames("/Users/zhuo/Desktop/frames");
        // let audio_fut = video_decoder.save_video_audio("/Users/zhuo/Desktop/audio.wav");

        // let (_res1, _res2) = tokio::join!(frames_fut, audio_fut);

        // let _ = video_decoder
        //     .save_video_frames("/Users/zhuo/Desktop/frames")
        //     .await;

        let metadata = video_decoder
            .get_video_metadata()
            .expect("failed to get video metadata");
        println!("{metadata:#?}");
    }
}

#[test_log::test(tokio::test)]
async fn test_save_video_segment() {
    #[cfg(feature = "ffmpeg-binary")]
    {
        let video_file = "/Users/xddotcom/Library/Application Support/ai.gendam.desktop/libraries/d3a13702-8f11-4dc6-86ea-42f63a92c3ad/files/fb6/fb62c84c5e20d5d0";
        let video_decoder = VideoDecoder::new(video_file).unwrap();
        let output_dir = "/Users/xddotcom/Downloads";
        let _result = video_decoder
            .save_video_segment(
                "test.mp4",
                output_dir,
                3000,
                5000,
            )
            .unwrap();
        // println!("{result:#?}");
    }
}
