use std::{path::Path};

use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use tracing::debug;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

pub struct AudioWhisper {
    ctx: WhisperContext,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WhisperItem {
    pub text: String,
    pub start_timestamp: i64,
    pub end_timestamp: i64,
}

impl AudioWhisper {
    pub async fn new(resources_dir: impl AsRef<Path>) -> anyhow::Result<Self> {
        let download = crate::download::FileDownload::new(crate::download::FileDownloadConfig {
            resources_dir: resources_dir.as_ref().to_path_buf(),
            ..Default::default()
        });

        let model_path = download
            .download_if_not_exists("whisper-ggml-base.bin")
            .await?
            .to_str()
            .ok_or(anyhow!("invalid path"))?
            .to_string();

        let ctx =
            WhisperContext::new_with_params(&model_path, WhisperContextParameters::default())?;

        debug!("context initialized");

        Ok(Self { ctx })
    }

    pub fn transcribe(
        &mut self,
        audio_file_path: impl AsRef<Path>,
    ) -> anyhow::Result<Vec<WhisperItem>> {
        let mut state = self.ctx.create_state()?;

        let mut params = FullParams::new(SamplingStrategy::default());

        // Edit params as needed.
        params.set_translate(true);

        // TODO actually we need to use auto language detection
        // however there is bug in whisper-rs

        // Disable anything that prints to stdout.
        params.set_print_special(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);

        // Open the audio file.
        let mut reader = hound::WavReader::open(audio_file_path).expect("failed to open file");
        #[allow(unused_variables)]
        let hound::WavSpec {
            channels,
            sample_rate,
            bits_per_sample,
            ..
        } = reader.spec();

        // Convert the audio to floating point samples.
        let mut audio = whisper_rs::convert_integer_to_float_audio(
            &reader
                .samples::<i16>()
                .map(|s| s.expect("invalid sample"))
                .collect::<Vec<_>>(),
        );

        // Convert audio to 16KHz mono f32 samples, as required by the model.
        // These utilities are provided for convenience, but can be replaced with custom conversion logic.
        // SIMD variants of these functions are also available on nightly Rust (see the docs).
        if channels == 2 {
            audio = whisper_rs::convert_stereo_to_mono_audio(&audio).map_err(|err| anyhow!(err))?;
        } else if channels != 1 {
            panic!(">2 channels unsupported");
        }

        if sample_rate != 16000 {
            panic!("sample rate must be 16KHz");
        }

        // Run the model.
        state.full(params, &audio[..])?;

        // Iterate through the segments of the transcript.
        let num_segments = state
            .full_n_segments()
            .expect("failed to get number of segments");

        let mut results = Vec::new();

        for i in 0..num_segments {
            // Get the transcribed text and timestamps for the current segment.
            let segment = state
                .full_get_segment_text(i)
                .expect("failed to get segment");
            let start_timestamp = state
                .full_get_segment_t0(i)
                .expect("failed to get start timestamp");
            let end_timestamp = state
                .full_get_segment_t1(i)
                .expect("failed to get end timestamp");

            results.push(WhisperItem {
                text: segment,
                start_timestamp,
                end_timestamp,
            });
        }

        Ok(results)
    }
}

#[test_log::test(tokio::test)]
async fn test_whisper() {
    let mut whisper =
        AudioWhisper::new("/Users/zhuo/dev/bmrlab/tauri-dam-test-playground/src-tauri/resources")
            .await
            .unwrap();
    match whisper
        .transcribe("/Users/zhuo/dev/bmrlab/tauri-dam-test-playground/src-tauri/.data/audio.wav")
    {
        Ok(result) => {
            for item in result {
                println!(
                    "[{}] [{}] {}",
                    item.start_timestamp, item.end_timestamp, item.text
                );
            }
        }
        Err(e) => {
            println!("{}", e);
        }
    }
}
