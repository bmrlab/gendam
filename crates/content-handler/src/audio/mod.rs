use crate::video::RawProbeOutput;
use anyhow::bail;
use byteorder::{LittleEndian, ReadBytesExt};
use content_metadata::audio::AudioMetadata;
use std::{
    io::{BufReader, Read},
    path::{Path, PathBuf},
};
use storage_macro::Storage;

const CHUNK_SIZE: usize = 1024 * 1024; // 1MB chunks

#[derive(Storage)]
pub struct AudioDecoder {
    file_path: PathBuf,
    ffmpeg_file_path: PathBuf,
    ffprobe_file_path: PathBuf,
}

impl AudioDecoder {
    pub fn new(file_path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let current_exe_path = std::env::current_exe().expect("failed to get current executable");
        let current_dir = current_exe_path
            .parent()
            .expect("failed to get parent directory");
        let ffmpeg_file_path = current_dir.join("ffmpeg");
        let ffprobe_file_path = current_dir.join("ffprobe");

        Ok(Self {
            file_path: file_path.as_ref().to_path_buf(),
            ffmpeg_file_path,
            ffprobe_file_path,
        })
    }

    pub fn get_audio_metadata(&self) -> anyhow::Result<AudioMetadata> {
        match std::process::Command::new(&self.ffprobe_file_path)
            .args([
                "-v",
                "error",
                "-show_streams",
                "-of",
                "json",
                self.file_path.to_str().expect("invalid audio file path"),
            ])
            .output()
        {
            Ok(output) => match String::from_utf8(output.stdout) {
                Ok(result) => {
                    let raw_output: RawProbeOutput = serde_json::from_str(&result)?;

                    match raw_output
                        .streams
                        .iter()
                        .find(|stream| stream.codec_type == "audio")
                    {
                        Some(stream) => Ok(AudioMetadata::from(stream)),
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
                bail!("Failed to get audio metadata: {e}");
            }
        }
    }

    pub fn generate_audio_waveform(&self, num_points: usize) -> anyhow::Result<Vec<f32>> {
        let metadata = self.get_audio_metadata()?;
        let total_samples = (metadata.duration * 44100.0) as usize;
        let samples_per_point = total_samples / num_points;

        let mut waveform = vec![0.0; num_points];
        let mut samples_processed = 0;
        let mut current_point = 0;
        let mut max_amplitude: f32 = 0.0;

        let mut ffmpeg = std::process::Command::new(&self.ffmpeg_file_path)
            .args(&[
                "-i",
                self.file_path.to_string_lossy().to_string().as_str(),
                "-f",
                "f32le",
                "-acodec",
                "pcm_f32le",
                "-ac",
                "1",
                "-ar",
                "44100",
                "-",
            ])
            .stdout(std::process::Stdio::piped())
            .spawn()?;

        let mut reader =
            BufReader::new(ffmpeg.stdout.take().expect("read data from ffmpeg stdout"));
        let mut buffer = [0u8; CHUNK_SIZE];

        loop {
            let bytes_read = reader.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }

            let samples = bytes_read / std::mem::size_of::<f32>();
            let mut sample_reader = &buffer[..bytes_read];

            for _ in 0..samples {
                let sample = sample_reader.read_f32::<LittleEndian>()?.abs();
                max_amplitude = max_amplitude.max(sample);
                samples_processed += 1;

                if samples_processed % samples_per_point == 0 {
                    waveform[current_point] = max_amplitude;
                    current_point += 1;
                    max_amplitude = 0.0;

                    if current_point >= num_points {
                        return Ok(waveform);
                    }
                }
            }
        }

        Ok(waveform)
    }

    pub fn save_audio_cover(&self, dest_path: impl AsRef<Path>) -> anyhow::Result<()> {
        let output_path = self.get_actual_path(dest_path.as_ref().to_path_buf())?;

        match std::process::Command::new(&self.ffmpeg_file_path)
            .args([
                "-i",
                self.file_path.to_str().expect("invalid audio file path"),
                "-an",
                "-vcodec",
                "copy",
                output_path.to_str().expect("invalid output path"),
            ])
            .output()
        {
            Ok(output) => {
                if !output.status.success() {
                    tracing::warn!("Failed to save audio cover");
                }
            }
            Err(e) => {
                tracing::warn!("Failed to save audio cover: {e}");
            }
        }

        Ok(())
    }

    pub fn save_whisper_format(&self, dest_path: impl AsRef<Path>) -> anyhow::Result<()> {
        let output_path = self.get_actual_path(dest_path.as_ref().to_path_buf())?;

        match std::process::Command::new(&self.ffmpeg_file_path)
            .args([
                "-i",
                self.file_path.to_str().expect("invalid audio file path"),
                "-vn",
                "-ar",
                // the rate must be 16KHz to fit whisper.cpp
                "16000",
                "-ac",
                "1",
                output_path.to_str().expect("invalid output path"),
            ])
            .output()
        {
            Ok(output) => {
                if !output.status.success() {
                    bail!("Failed to save whisper format");
                }
                Ok(())
            }
            Err(e) => {
                bail!("Failed to save whisper format: {e}");
            }
        }
    }
}
