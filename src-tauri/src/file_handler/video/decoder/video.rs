use super::{transcode::transcoder, utils};
use ffmpeg_next::ffi::*;
use std::path::Path;
use tracing::debug;

pub struct VideoDecoder {
    video_file_path: std::path::PathBuf,
}

impl VideoDecoder {
    pub fn new(filename: impl AsRef<Path>) -> Self {
        debug!("Successfully opened {}", filename.as_ref().display());

        let decoder = Self {
            video_file_path: filename.as_ref().to_path_buf(),
        };

        decoder
    }

    pub async fn save_video_frames(&self, frames_dir: impl AsRef<Path>) -> anyhow::Result<()> {
        save_video_frames(self.video_file_path.to_path_buf(), frames_dir)
    }

    pub async fn save_video_audio(&self, audio_path: impl AsRef<Path>) -> anyhow::Result<()> {
        save_video_audio(self.video_file_path.to_path_buf(), audio_path)
    }
}

fn save_video_frames(
    video_path: impl AsRef<Path>,
    frames_dir: impl AsRef<Path>,
) -> anyhow::Result<()> {
    let mut video = ffmpeg_next::format::input(&video_path.as_ref().to_path_buf())?;
    let video_stream = &video
        .streams()
        .best(ffmpeg_next::media::Type::Video)
        .ok_or(anyhow::anyhow!("no video stream found"))?;
    let video_stream_index = video_stream.index();

    let decoder_context = ffmpeg_next::codec::Context::from_parameters(video_stream.parameters())?;
    let mut decoder = decoder_context.decoder().video()?;

    // resize to make max size to 768
    let (target_width, target_height) = if decoder.width() > decoder.height() {
        (
            768,
            (768.0 * decoder.height() as f32 / decoder.width() as f32) as u32,
        )
    } else {
        (
            (768.0 * decoder.width() as f32 / decoder.height() as f32) as u32,
            768,
        )
    };

    let mut scaler = ffmpeg_next::software::scaling::context::Context::get(
        decoder.format(),
        decoder.width(),
        decoder.height(),
        AVPixelFormat::AV_PIX_FMT_RGB24.into(),
        target_width,
        target_height,
        ffmpeg_next::software::scaling::flag::Flags::BICUBIC,
    )?;

    for (stream, packet) in video.packets() {
        if stream.index() == video_stream_index {
            if decoder.send_packet(&packet).is_ok() {
                let mut frame = ffmpeg_next::frame::Video::empty();
                decoder.receive_frame(&mut frame)?;

                if frame.is_key() {
                    let mut scaled_frame = ffmpeg_next::frame::Video::empty();
                    scaler.run(&mut frame, &mut scaled_frame).unwrap();
                    let frames_dir = frames_dir.as_ref().to_path_buf().clone();

                    utils::copy_frame_props(&frame, &mut scaled_frame);
                    let array = utils::convert_frame_to_ndarray_rgb24(&mut scaled_frame).expect("");
                    let image = utils::array_to_image(array);
                    let _ = image.save(frames_dir.join(format!(
                        "{}.png",
                        scaled_frame.timestamp().unwrap().to_string()
                    )));
                }
            }
        }
    }

    Ok(())
}

fn save_video_audio(
    video_path: impl AsRef<Path>,
    audio_path: impl AsRef<Path>,
) -> anyhow::Result<()> {
    let mut video = ffmpeg_next::format::input(&video_path.as_ref().to_path_buf())?;
    let mut inner_output = ffmpeg_next::format::output(&audio_path)?;

    let video_stream = video
        .streams()
        .best(ffmpeg_next::media::Type::Audio)
        .unwrap();
    let audio_stream_index = video_stream.index();

    let mut transcoder = transcoder(&mut video, &mut inner_output, &audio_path, "anull")?;

    inner_output.set_metadata(video.metadata().to_owned());
    inner_output.write_header()?;

    for (stream, mut packet) in video.packets() {
        if stream.index() == audio_stream_index {
            packet.rescale_ts(stream.time_base(), transcoder.in_time_base);
            transcoder.send_packet_to_decoder(&packet);
            transcoder.receive_and_process_decoded_frames(&mut inner_output);
        }
    }

    transcoder.send_eof_to_decoder();
    transcoder.receive_and_process_decoded_frames(&mut inner_output);

    transcoder.flush_filter();
    transcoder.get_and_process_filtered_frames(&mut inner_output);

    transcoder.send_eof_to_encoder();
    transcoder.receive_and_process_encoded_packets(&mut inner_output);

    inner_output.write_trailer().unwrap();

    Ok(())
}

#[test_log::test(tokio::test)]
async fn test_video_decoder() {
    let video_decoder =
        VideoDecoder::new("/Users/zhuo/Desktop/file_v2_f566a493-ad1b-4324-b16f-0a4c6a65666g 2.MP4");

    let frames_fut = video_decoder
        .save_video_frames(
            "/Users/zhuo/Library/Application Support/cc.musedam.local/1aaa451c0bee906e2d1f9cac21ebb2ef5f2f82b2f87ec928fc04b58cbceda60b/frames",
        );
    let audio_fut = video_decoder
        .save_video_audio(
            "/Users/zhuo/Library/Application Support/cc.musedam.local/1aaa451c0bee906e2d1f9cac21ebb2ef5f2f82b2f87ec928fc04b58cbceda60b/audio.wav",
        );

    let (_res1, _res2) = tokio::join!(frames_fut, audio_fut);
}
