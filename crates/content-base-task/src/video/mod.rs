pub mod audio;
pub mod frame;
pub mod frame_embedding;
pub mod thumbnail;
pub mod trans_chunk;
pub mod trans_chunk_sum;
pub mod trans_chunk_sum_embed;
pub mod transcript;

use crate::task::ContentTaskType;
use audio::VideoAudioTask;
use content_base_derive::ContentTask;
use frame::VideoFrameTask;
use frame_embedding::VideoFrameEmbeddingTask;
use storage_macro::Storage;
use strum_macros::{EnumIter, EnumString};
use thumbnail::VideoThumbnailTask;
use trans_chunk::VideoTransChunkTask;
use trans_chunk_sum::VideoTransChunkSumTask;
use trans_chunk_sum_embed::VideoTransChunkSumEmbedTask;
use transcript::VideoTranscriptTask;

#[derive(Clone, Debug, EnumIter, EnumString, strum_macros::Display, ContentTask, Storage)]
#[strum(serialize_all = "kebab-case")]
pub enum VideoTaskType {
    Thumbnail(VideoThumbnailTask),
    Frame(VideoFrameTask),
    FrameEmbedding(VideoFrameEmbeddingTask),
    Audio(VideoAudioTask),
    Transcript(VideoTranscriptTask),
    TransChunk(VideoTransChunkTask),
    TransChunkSum(VideoTransChunkSumTask),
    TransChunkSumEmbed(VideoTransChunkSumEmbedTask),
}

impl Into<ContentTaskType> for VideoTaskType {
    fn into(self) -> ContentTaskType {
        ContentTaskType::Video(self)
    }
}

#[cfg(test)]
mod test {
    use super::{frame::VideoFrameTask, thumbnail::VideoThumbnailTask};
    use crate::{
        video::{
            audio::VideoAudioTask, trans_chunk::VideoTransChunkTask,
            trans_chunk_sum::VideoTransChunkSumTask,
            trans_chunk_sum_embed::VideoTransChunkSumEmbedTask, transcript::VideoTranscriptTask,
        },
        ContentTask, ContentTaskType, FileInfo, TaskRecord,
    };
    use ai::{
        llm::{openai::OpenAI, LLM},
        text_embedding::OrtTextEmbedding,
        tokenizers::Tokenizer,
        whisper::Whisper,
        AIModel,
    };
    use content_base_context::ContentBaseCtx;
    use content_handler::video::VideoDecoder;
    use content_metadata::ContentMetadata;
    use global_variable::{init_global_variables, set_current};
    use std::{path::PathBuf, str::FromStr, sync::Arc};

    #[test_log::test(tokio::test)]
    async fn test_video() {
        init_global_variables!();
        // set storage root path
        set_current!("abcdefg".into(), "/Users/zhuo/Desktop".into());

        // the artifacts_dir is relative to the storage root
        let content_base = ContentBaseCtx::new("gendam-test-artifacts", "");

        // initialize AI models
        let whisper =
            AIModel::new(|| async { Whisper::new("/Users/zhuo/dev/tezign/bmrlab/gendam/apps/desktop/src-tauri/resources/whisper/ggml-medium-q5_0.bin").await }, None).expect("whisper initialized");
        let llm = AIModel::new(
            || async {
                Ok(LLM::OpenAI(
                    OpenAI::new(
                        "http://localhost:11434/v1",
                        "ollama",
                        "qwen2:7b-instruct-q4_0",
                    )
                    .expect(""),
                ))
            },
            None,
        )
        .expect("");
        let text_embedding = AIModel::new(
            || async {
                OrtTextEmbedding::new("/Users/zhuo/dev/tezign/bmrlab/gendam/apps/desktop/src-tauri/resources/stella-base-zh-v3-1792d/model_quantized.onnx", "/Users/zhuo/dev/tezign/bmrlab/gendam/apps/desktop/src-tauri/resources/stella-base-zh-v3-1792d/tokenizer.json").await
            },
            None,
        ).expect("");
        let tokenizer = Tokenizer::from_file("/Users/zhuo/dev/tezign/bmrlab/gendam/apps/desktop/src-tauri/resources/qwen2/tokenizer.json").expect("");

        // add models to ContentBaseCtx
        let content_base = content_base
            .with_audio_transcript(Arc::new(whisper), "whisper")
            .with_llm(Arc::new(llm), tokenizer, "qwen2")
            .with_text_embedding(Arc::new(text_embedding), "stella");

        let file_identifier = "abcdefghijklmn";

        let file_info = FileInfo {
            file_identifier: file_identifier.into(),
            file_path: PathBuf::from_str("/Users/zhuo/Desktop/测试视频/4月1日.mp4")
                .expect("str should be valid path"),
        };

        let video_decoder = VideoDecoder::new(&file_info.file_path).expect("video decoder built");
        let metadata = video_decoder.get_video_metadata().expect("got metadata");
        let metadata = ContentMetadata::Video(metadata);

        let mut task_record = TaskRecord::from_content_base(file_identifier, &content_base).await;
        task_record
            .set_metadata(&content_base, &metadata)
            .await
            .expect("set metadata");

        tracing::info!("metadata: {:?}", metadata);

        let tasks: Vec<ContentTaskType> = vec![
            VideoThumbnailTask.into(),
            VideoFrameTask.into(),
            VideoAudioTask.into(),
            VideoTranscriptTask.into(),
            VideoTransChunkTask.into(),
            VideoTransChunkSumTask.into(),
            VideoTransChunkSumEmbedTask.into(),
        ];
        for task in tasks {
            let result = task.run(&file_info, &content_base).await;
            tracing::info!("result: {:?}", result);
            assert!(result.is_ok());
        }
    }
}
