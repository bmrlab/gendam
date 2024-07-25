use anyhow::bail;
use content_metadata::{
    audio::AudioMetadata,
    video::{VideoAvgFrameRate, VideoMetadata},
};
use serde::{Deserialize, Serialize};
use std::{path::Path, process::Stdio};
use storage::add_tmp_suffix_to_path;
use storage_macro::Storage;

const FRAME_FILE_EXTENSION: &'static str = "jpg";

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

#[derive(Storage)]
pub struct VideoDecoder {
    video_file_path: std::path::PathBuf,
    binary_file_path: std::path::PathBuf,
    ffprobe_file_path: std::path::PathBuf,
}

impl VideoDecoder {
    pub fn new(filename: impl AsRef<Path>) -> anyhow::Result<Self> {
        let current_exe_path = std::env::current_exe().expect("failed to get current executable");
        let current_dir = current_exe_path
            .parent()
            .expect("failed to get parent directory");
        let binary_file_path = current_dir.join("ffmpeg");
        let ffprobe_file_path = current_dir.join("ffprobe");

        tracing::debug!("ffmpeg path {:?}", &binary_file_path);

        Ok(Self {
            video_file_path: filename.as_ref().to_path_buf(),
            binary_file_path,
            ffprobe_file_path,
        })
    }
}

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
                "-vf",
                "scale='if(gte(iw,ih)*sar,768,-1)':'if(gte(iw,ih)*sar, -1, 768)',select=eq(n\\,0)",
                "-vframes",
                "1",
                "-compression_level",
                "9",
                "-f",
                "image2pipe",
                "pipe:1",
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
        {
            Ok(output) => {
                if !output.status.success() {
                    bail!(
                        "Failed to save video thumbnail: {}",
                        String::from_utf8_lossy(&output.stderr)
                    );
                }
                self.write(thumbnail_path.as_ref().to_path_buf(), output.stdout.into())
                    .await?;
                Ok(())
            }
            Err(e) => {
                bail!("Failed to save video thumbnail: {e}");
            }
        }
    }

    pub async fn save_video_frames(&self, frames_dir: impl AsRef<Path>) -> anyhow::Result<()> {
        // 单独提取 timestamp 为 0 的帧
        let frame_0_path = frames_dir
            .as_ref()
            .join(format!("0.{}", FRAME_FILE_EXTENSION));
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
                "-f",
                "image2pipe",
                "pipe:1",
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
        {
            Ok(output) => {
                if !output.status.success() {
                    bail!(
                        "Failed to save video frames: {}",
                        String::from_utf8_lossy(&output.stderr)
                    );
                }
                self.write(frame_0_path, output.stdout.into()).await?;
            }
            Err(e) => {
                bail!("Failed to save video frames: {e}");
            }
        }

        let actual_frame_dir = self.get_actual_path(frames_dir.as_ref().to_path_buf())?;
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
                actual_frame_dir
                    .join(format!("%d000-tmp.{}", FRAME_FILE_EXTENSION))
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
                self.save_batch_framer(frames_dir).await?;
            }
            Err(e) => {
                bail!("Failed to save video frames: {e}");
            }
        }

        Ok(())
    }

    async fn save_batch_framer(&self, frames_dir: impl AsRef<Path>) -> Result<(), anyhow::Error> {
        let actual_frame_dir = self.get_actual_path(frames_dir.as_ref().to_path_buf())?;
        for entry in std::fs::read_dir(actual_frame_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                if let Some(file_name) = path.file_name() {
                    if let Some(name_str) = file_name.to_str() {
                        if name_str.contains("-tmp") {
                            let new_name = name_str.replace("-tmp", "");
                            let new_path = frames_dir.as_ref().join(new_name);
                            self.write(new_path, std::fs::read(path.clone())?.into())
                                .await?;
                            std::fs::remove_file(path)?;
                        }
                    }
                }
            }
        }
        Ok(())
    }

    pub async fn save_video_audio(&self, audio_path: impl AsRef<Path>) -> anyhow::Result<()> {
        let actual_path = self.get_actual_path(audio_path.as_ref().to_path_buf())?;
        let tmp_path = add_tmp_suffix_to_path!(&actual_path);
        tracing::debug!("tmp_path: {:?}", tmp_path);
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
                tmp_path.to_str().expect("invalid audio path"),
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

                let content = std::fs::read(tmp_path.clone());
                match content {
                    Ok(data) => {
                        if let Ok(()) = self
                            .write(audio_path.as_ref().to_path_buf(), data.into())
                            .await
                        {
                            // 删除临时文件
                            if let Err(e) = std::fs::remove_file(tmp_path) {
                                tracing::info!("Failed to remove tmp audio file: {:?}", e);
                            }
                        }
                    }
                    Err(e) => {
                        bail!("Failed to save video audio: {e}");
                    }
                }
            }
            Err(e) => {
                bail!("Failed to save video frames: {e}");
            }
        }

        Ok(())
    }
}
