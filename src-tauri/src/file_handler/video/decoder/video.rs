use ffmpeg_next::ffi::*;

use std::{fs, path::Path};
use tracing::debug;

use super::{transcode::transcoder, utils};

use anyhow::{anyhow, Result};

pub struct VideoDecoder {
    video: ffmpeg_next::format::context::Input,
    video_stream_index: Option<usize>,
    video_decoder: Option<ffmpeg_next::codec::decoder::Video>,
    video_scaler: Option<ffmpeg_next::software::scaling::context::Context>,
    audio_stream_index: Option<usize>,
}

impl VideoDecoder {
    pub fn new(filename: impl AsRef<Path>) -> Result<Self> {
        let video = ffmpeg_next::format::input(&filename)?;

        debug!("Successfully opened {}", filename.as_ref().display());

        let mut decoder = Self {
            video,
            video_stream_index: None,
            video_decoder: None,
            video_scaler: None,
            audio_stream_index: None,
        };

        decoder.initialize()?;

        Ok(decoder)
    }

    pub fn save_video_artifacts(
        &mut self,
        frames_dir: impl AsRef<Path>,
        audio_path: impl AsRef<Path>,
    ) -> Result<()> {
        // preprocessing for audio file
        let mut output = None;
        let mut audio_transcoder = None;

        if let Some(_) = self.audio_stream_index {
            let mut inner_output = ffmpeg_next::format::output(&audio_path).unwrap();

            audio_transcoder = Some(transcoder(
                &mut self.video,
                &mut inner_output,
                &audio_path,
                "anull",
            )?);

            inner_output.set_metadata(self.video.metadata().to_owned());
            inner_output
                .write_header()
                .expect("failed to write output header");

            output = Some(inner_output);
        }

        for (stream, mut packet) in self.video.packets() {
            // use transcoder to handle audio packet
            if let Some(transcoder) = audio_transcoder.as_mut() {
                if let Some(mut output) = output.as_mut() {
                    if stream.index() == transcoder.stream {
                        packet.rescale_ts(stream.time_base(), transcoder.in_time_base);
                        transcoder.send_packet_to_decoder(&packet);
                        transcoder.receive_and_process_decoded_frames(&mut output);
                    }
                }
            }

            // handle video packet
            if let Some(video_stream_index) = self.video_stream_index {
                if stream.index() == video_stream_index {
                    let video_decoder = self
                        .video_decoder
                        .as_mut()
                        .expect("failed to initialize video_decoder");
                    let video_scaler = self
                        .video_scaler
                        .as_mut()
                        .expect("failed to initialize video_scaler");

                    match video_decoder.send_packet(&packet) {
                        Ok(()) => {
                            let mut frame = ffmpeg_next::frame::Video::empty();
                            match video_decoder.receive_frame(&mut frame) {
                                Ok(()) => {
                                    let mut scaled_frame = ffmpeg_next::frame::Video::empty();
                                    video_scaler.run(&mut frame, &mut scaled_frame).unwrap();
                                    utils::copy_frame_props(&frame, &mut scaled_frame);

                                    if scaled_frame.is_key() {
                                        // save frame to dir
                                        let array = utils::convert_frame_to_ndarray_rgb24(
                                            &mut scaled_frame,
                                        )
                                        .expect("failed to convert frame");

                                        let image = utils::array_to_image(array);
                                        image.save(frames_dir.as_ref().join(format!(
                                                "{}.png",
                                                scaled_frame
                                                    .timestamp()
                                                    .ok_or(anyhow!("frame has no timestamp"))?
                                                    .to_string()
                                            )))?;
                                    }
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        // postprocessing for audio file
        if let Some(transcoder) = audio_transcoder.as_mut() {
            if let Some(mut output) = output {
                transcoder.send_eof_to_decoder();
                transcoder.receive_and_process_decoded_frames(&mut output);

                transcoder.flush_filter();
                transcoder.get_and_process_filtered_frames(&mut output);

                transcoder.send_eof_to_encoder();
                transcoder.receive_and_process_encoded_packets(&mut output);

                output.write_trailer().unwrap();
            };
        };

        Ok(())
    }

    fn initialize(&mut self) -> Result<()> {
        let video_stream = self
            .video
            .streams()
            .best(ffmpeg_next::media::Type::Video)
            .ok_or(anyhow::anyhow!("no video stream found"))?;
        self.video_stream_index = Some(video_stream.index());

        // initialize video decoder and scaler
        match ffmpeg_next::codec::Context::from_parameters(video_stream.parameters()) {
            Ok(decoder_context) => match decoder_context.decoder().video() {
                Ok(decoder) => {
                    match ffmpeg_next::software::scaling::context::Context::get(
                        decoder.format(),
                        decoder.width(),
                        decoder.height(),
                        AVPixelFormat::AV_PIX_FMT_RGB24.into(),
                        decoder.width(),
                        decoder.height(),
                        ffmpeg_next::software::scaling::flag::Flags::BICUBIC,
                    ) {
                        Ok(scaler) => {
                            self.video_scaler = Some(scaler);
                        }
                        _ => {}
                    }

                    self.video_decoder = Some(decoder);
                }
                _ => {}
            },
            _ => {}
        }

        self.audio_stream_index = self
            .video
            .streams()
            .best(ffmpeg_next::media::Type::Audio)
            .map(|stream| stream.index());

        Ok(())
    }
}

#[test_log::test]
fn test_video_decoder() {
    todo!()
}
